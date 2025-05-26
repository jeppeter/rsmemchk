#! /usr/bin/env python

import extargsparse
import sys
import socket
import logging
import re
import os


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
		self.maps = []
		return

	def append_map(self,saddr, eaddr, mapfile):
		mp = MemoryMap(saddr,eaddr,mapfile)
		self.maps.append(mp)
		return
	def search_addr(self,addr):
		for m in self.maps:
			if addr >= m.startaddr and addr <= m.endaddr:
				return '%s +0x%x'%(m.mapfile,addr - m.startaddr)
		return None



def memlistparse_handler(args,parser):
	set_logging(args)
	fd = ReadFileLarge(args.input)
	rsmallocexpr = re.compile('^\\[RSMALLOC\\].*',re.I)
	memlistexpr = re.compile('.*memlist.*alignptr\\[([^\\]]+)\\]\\s+realptr\\[([^\\]]+)\\]\\s+size\\s+\\[([^\\]]+)\\].*callstack\\[([^\\]]+)\\]',re.I)
	deallocexpr = re.compile('.*deallocate:\\s+alignptr\\[([^\\]]+)\\]\\s+realptr\\[([^\\]]+)\\]',re.I)
	mapexpr = re.compile('.*memorymap\\[([0-9]+)\\]\\s+\\[([^\\]]+)\\]\\s+\\-\\s+\\[([^\\]]+)\\]\\s+\\[([^\\]]+)\\]',re.I)
	lindex = 0
	memleak = dict()
	meminfo = MemoryInfo()
	memlistafter = False
	for l in fd.fh:
		lindex += 1
		l = l.rstrip('\r\n')
		if rsmallocexpr.match(l):
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
	if len(memleak.keys()) > 0:
		# now to search for call stack
		for k in memleak.keys():
			curleak = memleak[k]
			sys.stdout.write('alignptr[0x%x]realptr[0x%x]size[0x%x]\n'%(curleak.alignptr,curleak.realptr,curleak.size))
			for fnaddr in memleak[k].callstacks:
				m = meminfo.search_addr(fnaddr)
				sys.stdout.write('    %s\n'%(m))
	sys.exit(0)


def main():
    commandline='''
    {
        "input|i" : null,
        "output|o" : null,
        "srcdir|S" : null,
        "memlistparse<memlistparse_handler>##to dump code in memlist##" : {
        	"$" : 0
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
