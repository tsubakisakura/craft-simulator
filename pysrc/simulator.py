import os
import sys
import subprocess

import tools

def run_simulator():
    env = os.environ.copy()
    env["MYSQL_PASSWORD"]=tools.get_mysql_password()
    cmdline = ["target/release/craft-simulator"] + sys.argv[1:]
    proc = subprocess.run(cmdline, stdout=sys.stdout, env=env)
    if proc.returncode != 0:
        print("Simulator has exit with error code:" + str(proc.returncode))
        return False

    return True

def main():
    tools.with_ssh_tunnel( lambda: run_simulator() )

if __name__ == '__main__':
    main()
