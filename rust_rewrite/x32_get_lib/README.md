# x32_get_lib

`x32_get_lib` is a command-line tool for saving preset libraries from a Behringer X32 or Midas M32 mixer to a local directory.

## Usage

```
x32_get_lib --ip <IP_ADDRESS> --directory <DIRECTORY> --library-type <LIBRARY_TYPE>
```

### Arguments

* `--ip <IP_ADDRESS>`: The IP address of the X32/M32 mixer.
* `--port <PORT>`: The local port to use for communication with the mixer (default: 10024).
* `--remote-port <REMOTE_PORT>`: The remote port of the X32/M32 mixer (default: 10023).
* `--directory <DIRECTORY>`: The directory to save the preset library to.
* `--library-type <LIBRARY_TYPE>`: The type of library to save. Possible values are:
    * `all`: Save all available libraries.
    * `channel`: Save channel presets.
    * `effects`: Save effects presets.
    * `routing`: Save routing presets.

## Example

```
x32_get_lib --ip 192.168.1.10 --directory ./presets --library-type all
```
