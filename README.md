# Macro-KB
This is a utility for creating a macro keyboard on linux. Any improvements/suggestions are welcome!

# Usage
You need to edit the main source file to contain the correct details (your username, the tmux session name, the bash handler script path) and you must have a tmux session start up automatically with the specified name to have it work.
There is also an example systemd service file in `systemd/system` directory for automatically starting the daemon that grabs the keyboard input. An example bash handler script is also provided as `example_handler.sh`

