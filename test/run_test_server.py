import itertools
import pathlib
import shutil
import subprocess
import sys
import xml.etree.ElementTree as XmlET


SCRIPT_DIR = pathlib.Path(__file__).parent
PKG_CFG = SCRIPT_DIR / "package_config.xml"
AS_SCRIPT = SCRIPT_DIR / "start_marshalrwr_test.as"
RWR_ROOT = pathlib.Path(r"C:\Program Files (x86)\Steam\steamapps\common\RunningWithRifles")
RWR_SERV = RWR_ROOT / "rwr_server.exe"
PKGS_ROOT = RWR_ROOT / "media/packages"


def _consume_prompt(proc):
    # consume the prompt marker
    prompt_marker = proc.stdout.read(1)
    if prompt_marker != ">":
        raise Exception(f"prompt marker read got unexpected '{prompt_marker}' :/")


def wait_for_server_load(proc):
    _spinner_steps = itertools.cycle(["/", "-", "\\", "|"])
    while True:
        # read a line from rwr server stdout
        _output_line = proc.stdout.readline().lstrip(">")
        # strip the line for easier processing
        _stripped_line = _output_line.strip()
        if _stripped_line.startswith("Loading"):
            _s = next(_spinner_steps)
            print(f"{_s} Loading...", end="\r")
        elif _stripped_line == "Game loaded":
            # exit the while loop now
            break
        else:
            print(_stripped_line)
    return True


def send_command(proc, cmd: str):
    proc.stdin.write(f"{cmd}\n")
    proc.stdin.flush()


def setup_test_pkg(test_pkg_dir, name, port, register):
    if not test_pkg_dir.exists():
        test_pkg_dir.mkdir()
    # copy in the latest package config
    shutil.copy(PKG_CFG, test_pkg_dir)
    (test_pkg_dir / "scripts").mkdir(exist_ok=True)
    # load the server start script
    mrwr_script_as = AS_SCRIPT.read_text()
    mrwr_script_as = mrwr_script_as.format(name=name, port=port, register=register)
    (test_pkg_dir / "scripts/start_marshalrwr_test.as").write_text(mrwr_script_as)


def cleanup_test_pkg(test_pkg_dir):
    # delete the test dir
    shutil.rmtree(test_pkg_dir, ignore_errors=True)
    pass


if __name__ == '__main__':
    name, port, register = sys.argv[1], sys.argv[2], sys.argv[3]
    print(f"Running test server 'MRWR_{name}' on port {port}")

    # set up the test env
    test_pkg_dir = PKGS_ROOT / f"_marshalrwr_{name}_test_pkg"
    setup_test_pkg(test_pkg_dir, name, port, register)
    # start an rwr server
    path_to_package = f"media/packages/{test_pkg_dir.name}"
    print(f"Starting RWR server for '{path_to_package}' package...")
    rwr_serv_args = [f"{RWR_SERV}"]
    rwr_serv = subprocess.Popen(rwr_serv_args, cwd=RWR_ROOT.absolute(), encoding="utf-8",
                                stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    try:
        # wait for the first prompt
        wait_for_server_load(rwr_serv)
        _consume_prompt(rwr_serv)
        start_server_command = f"start_script {AS_SCRIPT.name} {path_to_package}"
        print(f"Server loaded, sending '{start_server_command}'...")
        # write the start script command to rwr server stdin
        send_command(rwr_serv, start_server_command)
        # wait again as the server now loads from overlays set in the script
        wait_for_server_load(rwr_serv)
        print(f"Package script start completed!")
        # wait until Ctrl-C
        while True:
            # read a line from rwr server stdout
            output_line = rwr_serv.stdout.readline().lstrip(">")
            # strip the line for easier processing
            stripped_line = output_line.strip()
            print(stripped_line)
    except KeyboardInterrupt:
        print("Ctrl-C detected, shutting down...")
        print("Killing rwr server!")
        rwr_serv.kill()
        print("Cleaning up...")
        cleanup_test_pkg(test_pkg_dir)
