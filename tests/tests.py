import subprocess
import sys
import os
import random
import string
import shutil
import secrets
import time
import hashlib

MAX_ITERS = 100
MSG_LEN = 128 * 1024


TYPE = "release"

# Get the directory where the script lies
script_path = os.path.abspath(__file__)
script_dir = os.path.dirname(script_path)


ALICE_DIR = f"{script_dir}/env/sender_alice"
BOB_DIR = f"{script_dir}/env/receiver_bob"
ENV_DIRS = [ALICE_DIR, BOB_DIR]

def spawn_sender(cwd):
	os.makedirs(cwd, exist_ok=True)
	os.chdir(cwd)
	# Define the command to run, including any arguments
	command = [f'../../../target/{TYPE}/audibro', '--seed=40', '--key-charges=3', '--max-piece-size=10485760', '--config=../../../config.toml', 'sender', '0.0.0.0:5555', 'alice']

	# Start the subprocess and redirect stdin/stdout to pipes
	process = subprocess.Popen(command, cwd=cwd, stdin=subprocess.PIPE, stdout=subprocess.PIPE)
	return process

def spawn_receiver(cwd):
	os.makedirs(cwd, exist_ok=True)
	os.chdir(cwd)
	# Define the command to run, including any arguments
	command = [f'../../../target/{TYPE}/audibro', '--config=../../../config.toml',  'receiver', '127.0.0.1:5555', 'alice']


	# Start the subprocess and redirect stdin/stdout to pipes
	process = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE)
	return process


def clear_env(dirs):
	for directory in dirs:
		if os.path.exists(directory):
			# Remove the directory recursively
			shutil.rmtree(directory)
	print("Environemnt dirs cleared.")
			
def sha256(string):
	hasher = hashlib.sha256()
	hasher.update(string.encode())
	return hasher.digest().hex()

def test_scenarios(dir):
	# Iterate over all files in `dir`
	for file in os.listdir(dir):
		clear_env(ENV_DIRS)

		# Check whether file is in the desired format
		if file.endswith(".txt"):
			print("Testing scenario: ", file)
			with open(os.path.join(dir, file), 'r') as f:

				ps_alice = spawn_sender(ALICE_DIR)
				ps_bob = None

				seq = 1
				for exp_output in f:
					input = f"{seq}\r\n".encode()
					toks = exp_output.split(';')
					if toks[0] == 'skip':
						if ps_bob is not None:
							print(f'[{seq}] kill')
							ps_bob.terminate()
							ps_bob = None
							time.sleep(1)

						ps_alice.stdin.write(input)
						ps_alice.stdin.flush()
						time.sleep(0.5)
					else:
						if ps_bob is None:
							time.sleep(1)
							print(f'[{seq}] spawn')
							ps_bob = spawn_receiver(BOB_DIR)
							time.sleep(1)

						ps_alice.stdin.write(input)
						ps_alice.stdin.flush()
						time.sleep(0.5)
						act_output = ps_bob.stdout.readline().decode()
						#print(f"{act_output}")
						assert act_output == exp_output, "Message mismatch!"

					seq += 1

				if ps_alice is not None:
					ps_alice.terminate()
				if ps_bob is not None:
					ps_bob.terminate()
				time.sleep(1)
				print(f"Scenario '{file}' paased")

# Run the function as the main program
if __name__ == '__main__':
	clear_env(ENV_DIRS)

	test_scenarios(f"{script_dir}/scenarios/")
	ps_alice = spawn_sender(ALICE_DIR)
	ps_bob = spawn_receiver(BOB_DIR)
	time.sleep(1)
	alphabet = string.ascii_letters + string.digits
