import subprocess
import sys
import os
import random
import string
import shutil
import secrets
import time

#TYPE = "debug"
TYPE = "release"

def spawn_sender(cwd):
    # Define the command to run, including any arguments
    command = [f'../target/{TYPE}/audibro', '--seed=40', '--layers=3', '--key-lifetime=3', '--max-piece-size=10485760', '--config=../../../config.toml', 'sender', '0.0.0.0:5555', 'alice']
    os.makedirs(cwd, exist_ok=True)

    # Start the subprocess and redirect stdin/stdout to pipes
    process = subprocess.Popen(command, cwd=cwd, stdin=subprocess.PIPE, stdout=subprocess.PIPE)
    return process

def spawn_receiver(cwd):
    # Define the command to run, including any arguments
    command = [f'../target/{TYPE}/audibro', '--config=../../../config.toml',  'receiver', '127.0.0.1:5555', 'alice']

    os.makedirs(cwd, exist_ok=True)

    # Start the subprocess and redirect stdin/stdout to pipes
    process = subprocess.Popen(command, cwd=cwd, stdin=subprocess.PIPE, stdout=subprocess.PIPE)
    return process


def clear_env(dirs):
    for directory in dirs:
        if os.path.exists(directory):
            # Remove the directory recursively
            shutil.rmtree(directory)
    print("Environemnt dirs cleared.")
            

# Run the function as the main program
if __name__ == '__main__':

    MAX_ITERS = 1000
    MSG_LEN = 1024
    ALICE_DIR = "env/sender_alice"
    BOB_DIR = "env/receiver_bob"
    ENV_DIRS = [ALICE_DIR, BOB_DIR]

    clear_env(ENV_DIRS)
    
    ps_alice = spawn_sender(ALICE_DIR)
    ps_bob = spawn_receiver(BOB_DIR)

    time.sleep(1)

    alphabet = string.ascii_letters + string.digits
    random_string = ''.join(secrets.choice(alphabet) for _ in range(MSG_LEN))
    random_string += '\n'

    iter = 0
    while iter < MAX_ITERS:
        print("Iteration: ", iter)

        #random_string = secrets.choice(alphabet) + random_string[1:]

        ps_alice.stdin.write(random_string.encode())
        ps_alice.stdin.flush()
        toks = ps_bob.stdout.readline().decode().split(';')
        assert toks[3] == random_string, "Message mismatch!"

        iter += 1
