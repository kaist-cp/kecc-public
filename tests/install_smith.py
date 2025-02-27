import subprocess, os


def execute_command(command, cwd=None):
    try:
        process = subprocess.Popen(
            command, shell=True, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE
        )
        stdout, stderr = process.communicate()

        if process.returncode != 0:
            raise subprocess.CalledProcessError(
                process.returncode, command, stderr.decode("utf-8")
            )
        print(stdout.decode("utf-8"))
        return True
    except subprocess.CalledProcessError as e:
        print(f"Error executing command: {e.cmd}")
        print(e.output)
        return False
    except Exception as e:
        print(f"Unexpected error: {str(e)}")
        return False


def install_csmith():
    """
    Installation based on the provided in the github repos
    of the package. Make sure to have sudo privileges.
    """
    usr_bin_path = "/usr/local/bin/csmith"
    usr_inc_path = "/usr/local/include/"  # cmake dumps the include files here.
    if os.path.exists(usr_bin_path):
        return usr_bin_path, usr_inc_path

    csmith_dir = os.path.join(os.getcwd(), "csmith")
    if not os.path.exists(csmith_dir):
        if not execute_command(
            "git clone https://github.com/csmith-project/csmith.git"
        ):
            raise Exception("Unable to clone the Csmith repository")

    if not execute_command("sudo apt install -y g++ cmake m4", cwd=csmith_dir):
        raise Exception("Unable to install dependencies")

    cmake_command = f"cmake -DCMAKE_INSTALL_PREFIX=/usr/local/ ."
    if not execute_command(cmake_command, cwd=csmith_dir):
        raise Exception("Unable to run cmake.")

    if not execute_command("make && sudo make install", cwd=csmith_dir):
        raise Exception("Unable to install.")
    return usr_bin_path, usr_inc_path


if __name__ == "__main__":
    install_csmith()
