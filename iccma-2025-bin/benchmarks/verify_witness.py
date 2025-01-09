"""
Copyright <2023> <Andreas Niskanen, University of Helsinki>

Permission is hereby granted, free of charge, to any person obtaining a copy of this
software and associated documentation files (the "Software"), to deal in the Software
without restriction, including without limitation the rights to use, copy, modify,
merge, publish, distribute, sublicense, and/or sell copies of the Software, and to
permit persons to whom the Software is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice shall be included in all copies
or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT
OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
OTHER DEALINGS IN THE SOFTWARE.
"""


"""

!!!
!!! IMPORTANT!!!
!!!

This file is NOT the original from ICCMA 2023. 
It has been slightly altered and is only used to test our solver implementation with Benchmarks from ICCMA 2023

!!!
!!! IMPORTANT!!!
!!!

"""


import sys
from pysat.formula import CNF
from pysat.formula import IDPool
from pysat.solvers import Glucose4 as Solver

task = sys.argv[1]
af_file = sys.argv[2]
out_file = sys.argv[3]

# if timeout or memout there is nothing to verify
# contents = open(out_file.replace(".out", ".var")).read().split("\n")
# contents = [line for line in contents if not line.startswith("#") and len(line) > 0]
# assert(contents[-3].startswith("TIMEOUT=") and contents[-2].startswith("MEMOUT="))
# contents = [line.split("=")[1] for line in contents]
# if contents[-3] == "true" or contents[-2] == "true":
# 	print("PASS (timeout)")
# 	sys.exit(0)

# read the output file and extract potential witness
out_file_contents = open(out_file).read().split("\n")
out_file_contents = [line.strip() for line in out_file_contents if len(line) > 0]
witness = None
if any(line.startswith("w") for line in out_file_contents):
	witness_lines = [line for line in out_file_contents if line.startswith("w")]
	assert(len(witness_lines) == 1)
	witness = list(map(int, witness_lines[0].replace("w", "").strip().split()))
answer = "NA"
if "YES" in out_file_contents:
	answer = "YES"
if "NO" in out_file_contents:
	answer = "NO"

problem, semantics = task.split("-")

# witness must exist in the following cases
if problem == "SE":
	if semantics != "ST" or answer != "NO":
		assert(witness is not None)
	else:
		print("PASS (answer is NO)")
		sys.exit(0)
if problem == "DC":
	if answer == "YES":
		assert(witness is not None)
	else:
		print("PASS (answer is NO)")
		sys.exit(0)
if problem == "DS":
	if answer == "NO":
		assert(witness is not None)
	else:
		print("PASS (answer is YES)")
		sys.exit(0)

# check that query is in witness
if problem == "DC":
	query = int(open(af_file + ".arg").read().strip())
	assert(query in witness)

# or that query is not in witness
if problem == "DS":
	query = int(open(af_file + ".arg").read().strip())
	assert(query not in witness)

# read the original af
af_file_contents = open(af_file).read().split("\n")
af_file_contents = [line.strip() for line in af_file_contents if not line.startswith("#") and len(line) > 0]
p_line = af_file_contents[0]
attack_lines = af_file_contents[1:]
arguments = range(1, int(p_line.replace("p af ", ""))+1)
n = len(arguments)
attacks = [tuple(map(int, line.split())) for line in attack_lines]
attackers = { a : [] for a in arguments }
for a,b in attacks:
	attackers[b].append(a)

witness = set(witness)
in_witness = sorted([a for a in witness])
out_witness = sorted([a for a in arguments if a not in witness])
assumptions = in_witness + [-a for a in out_witness]

def cf_encoding(solver):
	for a,b in attacks:
		solver.add_clause([-a, -b])

def out_encoding(solver):
	for a in arguments:
		clause = [-(n+a)]
		for b in attackers[a]:
			solver.add_clause([-b, n+a])
			clause.append(b)
		solver.add_clause(clause)

def adm_encoding(solver):
	cf_encoding(solver)
	out_encoding(solver)
	for a in arguments:
		for b in attackers[a]:
			solver.add_clause([-a, n+b])

def com_encoding(solver):
	adm_encoding(solver)
	for a in arguments:
		clause = [a]
		for b in attackers[a]:
			clause.append(-(n+b))
		solver.add_clause(clause)

def stb_encoding(solver):
	cf_encoding(solver)
	out_encoding(solver)
	for a in arguments:
		solver.add_clause([a, n+a])

# verify witness
solver = Solver(with_proof=True)
if semantics == "CO":
	com_encoding(solver)
	assert(solver.solve(assumptions))
elif semantics == "ST":
	stb_encoding(solver)
	assert(solver.solve(assumptions))
elif semantics == "PR":
	# witness is a complete extension
	com_encoding(solver)
	assert(solver.solve(assumptions))
	# no superset is complete
	for a in in_witness:
		solver.add_clause([a])
	solver.add_clause([a for a in out_witness])
	assert(not solver.solve())
elif semantics == "SST":
	# witness is a complete extension
	com_encoding(solver)
	for a in arguments:
		solver.add_clause([-(2*n+a), a, n+a])
		solver.add_clause([2*n+a, -a])
		solver.add_clause([2*n+a, -(n+a)])
	assert(solver.solve(assumptions))
	# extract range of witness
	model = solver.get_model()
	in_range = [a for a in arguments if model[2*n+a-1] > 0]
	out_range = [a for a in arguments if model[2*n+a-1] < 0]
	# no range-superset is complete
	for a in in_range:
		solver.add_clause([2*n+a])
	solver.add_clause([2*n+a for a in out_range])
	assert(not solver.solve())
elif semantics == "STG":
	# witness is a conflict-free extension
	cf_encoding(solver)
	out_encoding(solver)
	for a in arguments:
		solver.add_clause([-(2*n+a), a, n+a])
		solver.add_clause([2*n+a, -a])
		solver.add_clause([2*n+a, -(n+a)])
	assert(solver.solve(assumptions))
	# extract range of witness
	model = solver.get_model()
	in_range = [a for a in arguments if model[2*n+a-1] > 0]
	out_range = [a for a in arguments if model[2*n+a-1] < 0]
	# no range-superset is conflict-free
	for a in in_range:
		solver.add_clause([2*n+a])
	solver.add_clause([2*n+a for a in out_range])
	assert(not solver.solve())
elif semantics == "ID":
	pass
else:
	sys.exit(1)

print("PASS (all tests passed)")

if semantics == "PR" or semantics == "SST" or semantics == "STG":
	proof_file = open(out_file.replace(".out", ".drup"), "w")
	for line in solver.get_proof():
		proof_file.write(line + "\n")
	proof_file.close()
