#! /usr/bin/env python

import extargsparse
import sys
import socket
import logging
import re
import os
import traceback
import struct
from rust_demangler import demangle
import pefile


class ReadFileLarge(object):
    def __init__(self,fname=None):
        self.fname = fname
        if fname is not None:
            self.fh = open(fname,'r')
        else:
            self.fh = sys.stdin
        self.readb = b''
        self.sidx = 0
        self.linenum = 0
        return


def set_logging(args):
    loglvl= logging.ERROR
    if args.verbose >= 3:
        loglvl = logging.DEBUG
    elif args.verbose >= 2:
        loglvl = logging.INFO
    curlog = logging.getLogger(args.lognames)
    #sys.stderr.write('curlog [%s][%s]\n'%(args.logname,curlog))
    curlog.setLevel(loglvl)
    if len(curlog.handlers) > 0 :
        curlog.handlers = []
    formatter = logging.Formatter('%(asctime)s:%(filename)s:%(funcName)s:%(lineno)d<%(levelname)s>\t%(message)s')
    if not args.lognostderr:
        logstderr = logging.StreamHandler()
        logstderr.setLevel(loglvl)
        logstderr.setFormatter(formatter)
        curlog.addHandler(logstderr)

    for f in args.logfiles:
        flog = logging.FileHandler(f,mode='w',delay=False)
        flog.setLevel(loglvl)
        flog.setFormatter(formatter)
        curlog.addHandler(flog)
    for f in args.logappends:       
        if args.logrotate:
            flog = logging.handlers.RotatingFileHandler(f,mode='a',maxBytes=args.logmaxbytes,backupCount=args.logbackupcnt,delay=0)
        else:
            sys.stdout.write('appends [%s] file\n'%(f))
            flog = logging.FileHandler(f,mode='a',delay=0)
        flog.setLevel(loglvl)
        flog.setFormatter(formatter)
        curlog.addHandler(flog)
    return

def load_log_commandline(parser):
    logcommand = '''
    {
        "verbose|v" : "+",
        "logname" : "root",
        "logfiles" : [],
        "logappends" : [],
        "logrotate" : true,
        "logmaxbytes" : 10000000,
        "logbackupcnt" : 2,
        "lognostderr" : false
    }
    '''
    parser.load_command_line_string(logcommand)
    return parser

def parse_int(v):
    c = v
    base = 10
    if c.startswith('0x') or c.startswith('0X') :
        base = 16
        c = c[2:]
    elif c.startswith('x') or c.startswith('X'):
        base = 16
        c = c[1:]
    return int(c,base)


class MemLeak(object):
	def __init__(self,alignptr,realptr,size,callstks):
		self.alignptr = alignptr
		self.realptr = realptr
		self.size = size
		self.callstacks = callstks
		return

class MemoryMap(object):
	def __init__(self,saddr, eaddr,mapfile):
		self.startaddr = saddr
		self.endaddr = eaddr
		self.mapfile = mapfile
		return

class MemoryInfo(object):
	def __init__(self):
		self.addrs  = dict()
		self.filemap = dict()
		return

	def append_map(self,saddr, eaddr, mapfile):
		if mapfile not in self.filemap.keys():
			logging.info('create %s'%(mapfile))
			self.filemap[mapfile] = []
		mp = MemoryMap(saddr,eaddr,mapfile)
		logging.info('append %s [0x%x,0x%x]'%(mapfile,saddr,eaddr))
		self.filemap[mapfile].append(mp)
		return

	def calc_map(self):
		self.addrs = dict()
		for k in self.filemap.keys():
			mps = self.filemap[k]
			if len(mps) > 0:
				saddr = mps[0].startaddr
				eaddr = mps[-1].endaddr
				self.addrs[k] = MemoryMap(saddr,eaddr,k)

	def search_addr(self,addr):
		for k in self.addrs.keys():
			if addr >= self.addrs[k].startaddr and addr <= self.addrs[k].endaddr:
				#logging.info('search [%s]'%(k))
				offaddr = 0
				mps = self.filemap[k]
				idx = 0
				while idx < len(mps):
					if mps[idx].startaddr <= addr and mps[idx].endaddr >= addr:
						return k,(offaddr + addr - mps[idx].startaddr)
					offaddr += mps[idx].endaddr - mps[idx].startaddr + 1
					idx += 1
		return None,None



class ObjDumpAsm(object):
	def __init__(self,fname):
		self.fname = fname
		self.fd = ReadFileLarge(fname)
		self.addrmap = dict()		
		self.addrstart = []
		self.addrend = []
		self._parse_file()
		return

	def _parse_file(self):
		startexpr = re.compile('^([0-9a-fA-F]+)\\s+<([^>]+)>:',re.I)
		if self.fd is None:
			return
		for l in self.fd.fh:
			l = l.rstrip('\r\n')
			m = startexpr.findall(l)
			if m is not None and len(m) > 0:
				startstr = '0x%s'%(m[0][0])
				saddr = parse_int(startstr)
				if (len(self.addrend) >0 and saddr > self.addrend[-1]) or len(self.addrend) == 0:
					self.addrstart.append(saddr)
					if len(self.addrend) > 0:
						self.addrend[-1] = saddr - 1
						self.addrend.append(saddr)
					else:
						self.addrend.append(saddr + 1)
					nstr = '0x%x'%(saddr)
					logging.info('%s name %s'%(nstr, m[0][1]))
					self.addrmap[nstr] = m[0][1]
		if len(self.addrend) > 0:
			self.addrend[-1] = self.addrstart[-1] + 1
		del self.fd
		self.fd = None
		return

	def _return_addr_val(self,cidx,addr):
		addrs = '0x%x'%(self.addrstart[cidx])
		cs = None
		if addrs in self.addrmap.keys():
			cs = self.addrmap[addrs]
			try:
				cs = demangle(cs)
			except:
				pass
		if cs is not None:
			return '%s +0x%x'%(cs,addr - self.addrstart[cidx])
		return None


	def search_addr(self,addr):
		sidx = 0
		assert(len(self.addrstart) == len(self.addrend))
		eidx = len(self.addrstart) - 1
		cidx = int((sidx + eidx) >> 1)
		while sidx < eidx:
			if cidx == sidx:
				if self.addrstart[cidx] <= addr and self.addrend[cidx] >= addr:
					return self._return_addr_val(cidx,addr)
				elif self.addrend[cidx] < addr:
					sidx += 1
				else:
					eidx -= 1
			elif cidx == eidx:
				if self.addrstart[cidx] <= addr and self.addrend[cidx] >= addr:
					return self._return_addr_val(cidx,addr)
				elif self.addrstart[cidx] > addr:
					eidx -= 1
				else:
					sidx += 1
			else:
				if self.addrstart[cidx] <= addr and self.addrend[cidx] >= addr:
					return self._return_addr_val(cidx,addr)
				elif self.addrstart[cidx] > addr:
					eidx = cidx
				else:
					sidx = cidx
			cidx = int((sidx + eidx) >> 1)
		logging.info('cidx %d len(%d)'%(cidx,len(self.addrstart)))
		if self.addrstart[cidx] <= addr and self.addrend[cidx] >= addr:
			return self._return_addr_val(cidx,addr)
		return None



class ObjDumpMap(object):
	def __init__(self,srcdir):
		self.srcdir = srcdir
		self.objdumpmap = dict()
		return

	def parse_file(self,fname):
		bname = os.path.basename(fname)
		asmname = os.path.join(self.srcdir,'%s.asm'%(bname))
		#logging.info('will scan %s'%(asmname))
		if os.path.exists(asmname) and asmname not in self.objdumpmap.keys():
			self.objdumpmap[asmname] = ObjDumpAsm(asmname)
		return

	def search_addr(self,fname,addr):
		bname = os.path.basename(fname)
		asmname = os.path.join(self.srcdir,'%s.asm'%(bname))
		if os.path.exists(asmname) and asmname in self.objdumpmap.keys():
			return self.objdumpmap[asmname].search_addr(addr)
		return None

class PeMapTrans(object):
	def __init__(self,f):
		self.fname = f
		self.VirtualAddress = None
		self.PointerToRawData = None
		self.ImageBase = None
		return

	def parse_pe(self):
		try:
			pe = pefile.PE(self.fname)
			for s in pe.sections:
				if sys.version[0] == '3':
					nb = b''
					idx = 0
					while idx < len(s.Name):
						if s.Name[idx] == 0x0:
							break
						nb += struct.pack('B',s.Name[idx])
						idx += 1
					n = nb.decode('utf-8')
				else:
					n = str(s.Name)
				logging.info('n [%s]'%(n))
				logging.info('%s'%(s))
				if n == '.text':
					self.VirtualAddress = s.VirtualAddress
					self.PointerToRawData = s.PointerToRawData
					self.ImageBase = pe.OPTIONAL_HEADER.ImageBase
					return True
		except:
			logging.error('%s'%(traceback.format_exc()))
			return False
		logging.info('%s False'%(self.fname))
		return False

class PeMap(object):
	def __init__(self,srcdir):
		self.srcdir = srcdir
		self.petrans = dict()
		return

	def parse_pe(self,fname):
		bname = os.path.basename(fname)
		curfile = os.path.join(self.srcdir,bname)	
		logging.info('test %s'%(curfile))
		if os.path.exists(curfile) and (curfile.endswith('.exe') or curfile.endswith('.dll')):
			if bname not in self.petrans.keys():
				cb = PeMapTrans(curfile)
				retval = cb.parse_pe()
				if retval:
					logging.info('%s succ pe'%(bname))
					self.petrans[bname] = cb


def trans_pe_addr(pemap,fname,addr):
	bname = os.path.basename(fname)
	retaddr = addr
	if bname in pemap.petrans.keys():
		logging.info('find %s'%(bname))
		cb = pemap.petrans[bname]
		if cb.VirtualAddress is not None and  cb.PointerToRawData is not None:
			retaddr = addr + cb.ImageBase
	else:
		logging.info('no [%s]'%(bname))
	logging.info('trans pe addr 0x%x => 0x%x'%(addr,retaddr))
	return retaddr




def memlistparse_handler(args,parser):
	set_logging(args)
	fd = ReadFileLarge(args.input)
	rsmemchkexpr = re.compile('^\\[RSMEMCHK\\].*',re.I)
	memlistexpr = re.compile('.*memlist.*alignptr\\[([^\\]]+)\\]\\s+realptr\\[([^\\]]+)\\]\\s+size\\s+\\[([^\\]]+)\\].*callstack\\[([^\\]]+)\\]',re.I)
	deallocexpr = re.compile('.*deallocate:\\s+alignptr\\[([^\\]]+)\\]\\s+realptr\\[([^\\]]+)\\]',re.I)
	mapexpr = re.compile('.*memorymap\\[([0-9]+)\\]\\s+\\[([^\\]]+)\\]\\s+\\-\\s+\\[([^\\]]+)\\]\\s+\\[([^\\]]+)\\]',re.I)
	lindex = 0
	memleak = dict()
	meminfo = MemoryInfo()
	memlistafter = False
	for l in fd.fh:
		lindex += 1
		if (lindex % 1000) == 0:
			logging.info('%d'%(lindex))
		l = l.rstrip('\r\n')
		if rsmemchkexpr.match(l):
			# to test for the memlist
			if memlistafter:
				# to match 
				m = memlistexpr.findall(l)
				if m is not None and len(m) > 0:
					alignptr = parse_int(m[0][0])
					realptr= parse_int(m[0][1])
					size = parse_int(m[0][2])
					sarr = re.split(',',m[0][3])
					stks = []
					for c in sarr:
						stks.append(parse_int(c))
					memleak['0x%x'%(alignptr)] = MemLeak(alignptr,realptr,size,stks)
				else:
					m = deallocexpr.findall(l)
					if m is not None and len(m) > 0:
						alignptr = parse_int(m[0][0])
						realptr = parse_int(m[0][1])
						k = '0x%x'%(alignptr)
						if k in memleak.keys():
							logging.info('memleak %s deallocated'%(k))
							del memleak[k]
					else:
						m = mapexpr.findall(l)
						if m is not None and len(m) > 0:
							startaddr = parse_int(m[0][1])
							endaddr = parse_int(m[0][2])
							mapfile = m[0][3]
							meminfo.append_map(startaddr,endaddr,mapfile)
			else:
				m = memlistexpr.findall(l)
				if m is not None and len(m) > 0:
					memlistafter = True
					alignptr = parse_int(m[0][0])
					realptr= parse_int(m[0][1])
					size = parse_int(m[0][2])
					sarr = re.split(',',m[0][3])
					stks = []
					for c in sarr:
						stks.append(parse_int(c))
					memleak['0x%x'%(alignptr)] = MemLeak(alignptr,realptr,size,stks)

	if args.srcdir is not None:
		asmmap = ObjDumpMap(args.srcdir)
		pemap = PeMap(args.srcdir)
	if len(memleak.keys()) > 0:
		meminfo.calc_map()
		# now to search for call stack
		for k in memleak.keys():
			curleak = memleak[k]
			sys.stdout.write('alignptr[0x%x]realptr[0x%x]size[0x%x]\n'%(curleak.alignptr,curleak.realptr,curleak.size))
			for fnaddr in memleak[k].callstacks:
				fname,addr = meminfo.search_addr(fnaddr)
				if fname is not None:
					fnaddrs = ''
					if args.srcdir is not None:
						asmmap.parse_file(fname)
						if fname.endswith('.exe') or fname.endswith('.dll') :
							logging.info('pe format')
							pemap.parse_pe(fname)
							addr = trans_pe_addr(pemap,fname,addr)
						ns = asmmap.search_addr(fname,addr)
						if ns is not None:
							fnaddrs = ns
					sys.stdout.write('    %s +0x%x %s\n'%(fname,addr,fnaddrs))


	sys.exit(0)

def readpe_handler(args,parser):
	set_logging(args)
	for f in args.subnargs:
		try:
			pe = pefile.PE(f)
			sys.stdout.write('%s file\n'%(f))
			sys.stdout.write('    ImageBase 0x%x\n'%(pe.OPTIONAL_HEADER.ImageBase))
			for sec in pe.sections:
				if sys.version[0] == '3':
					n = sec.Name.decode('utf-8')
				else:
					n = str(sec.Name)
				sys.stdout.write('   name %s rawdata 0x%x VirtualAddress 0x%x\n'%(n,sec.PointerToRawData,sec.VirtualAddress))
		except:
			logging.error('%s'%(traceback.format_exc()))
	sys.exit(0)
	return

def parsepe_handler(args,parser):
	set_logging(args)
	pemap = PeMap(args.srcdir)
	for f in args.subnargs:
		cb = PeMapTrans(f)
		pemap.parse_pe(f)
	sys.exit(0)

def main():
    commandline='''
    {
        "input|i" : null,
        "output|o" : null,
        "srcdir|S" : null,
        "memlistparse<memlistparse_handler>##to dump code in memlist##" : {
        	"$" : 0
        },
        "readpe<readpe_handler>##file ... to parse pe##" : {
        	"$" : "+"
        },
        "parsepe<parsepe_handler>##file ... to parse pe##" : {
        	"$" : "+"
        }
    }
    '''
    parser = extargsparse.ExtArgsParse()
    parser.load_command_line_string(commandline)
    load_log_commandline(parser)
    parser.parse_command_line(None,parser)
    raise Exception('can not reach here')
    return

if __name__ == '__main__':
    main()
