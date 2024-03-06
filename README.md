# Rdp Config Update
Use to update the [Microsoft Remote Desktop](https://learn.microsoft.com/en-us/windows-server/remote/remote-desktop-services/clients/remote-desktop-clients) config.\
This is my first program written in rust, and I am still learning at this.

## How to install RDP
Look at [RDP Wrapper Library by Stas'M](https://github.com/stascorp/rdpwrap).\
And thanks to [RDPWrap.ini](https://github.com/sebaxakerhtc/rdpwrap.ini).

## Automatic Boot
You can use the [Task Scheduler](https://www.windowscentral.com/how-create-automated-task-using-task-scheduler-windows-10)

## Usage
This is a terminal program, you can use "-h" to check all functions.

| Param                       | Function                                              |
|-----------------------------|-------------------------------------------------------|
| -h / --help                 | Show the help list.                                   |
| -V / --version              | Show versions.                                        |
| -q / --quiet                | Silent mode execution (not waiting for user input).   |
| -u / --url \<URL>           | Update using the specified url.                       |
| -p / --position \<POSITION> | Specify the location of rdpwrap. ini.                 |
| --reboot                    | Restart the RDP service without checking for updates. |
