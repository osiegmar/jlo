# J'Lo – the Java Loader

The Java Loader (or J'Lo for short) is a minimalistic tool to download and manage Java installations on your machine.
It is written in Rust, with a main focus on simplicity and ease of use.

J'Lo currently supports Linux (x86_64) and macOS (arm64).

At the moment, only the Eclipse Temurin distribution is available.
Java versions are supported starting from Java 8, with all newer versions working automatically.

## Installing J'Lo

To install J'Lo on Unix-like systems (Linux, macOS, WSL, etc.):

```shell
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/java-loader/jlo/refs/heads/main/install.sh)"
```

This will download the latest J'Lo binary and install it to `~/.jlo/`.
You can safely re-run this command to update J'Lo to the latest version, which is basically the same as running
`jlo selfupdate`.

> [!IMPORTANT]
> Watch the output closely, as you will have to add some lines to your shell profile to make `jlo` available in your
> terminal.

## Quick Start

Setup environment for Java 25 (installing it first, if necessary):

```shell
# Update JAVA_HOME and PATH for Java 25 in the current shell session.
jlo env 25

# Optionally, verify that the correct Java version is being used:
echo $JAVA_HOME
which java
```

Alternatively:

```shell
cd /path/to/your/project

# One-time setup: create a .jlorc file that pins Java 25 for this project.
jlo init 25

# Setup JAVA_HOME and PATH for the Java version specified in the .jlorc file.
jlo env
```

> [!TIP]
> If you enabled the J'Lo autoload feature during installation, J'Lo will automatically set up the Java environment
> whenever you `cd` into a directory that contains a `.jlorc` file.

JDKs are installed to `~/.jdks/` on Linux and `~/Library/Java/JavaVirtualMachines/` on macOS.
This allows automatic discovery of installed JDKs by IDEs like IntelliJ IDEA.

## Command Reference

| Command          | Description                                                                                                                                                                                   |
|------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `jlo env`        | Set up the environment for the Java version specified in the `.jlorc` file.                                                                                                                   |
| `jlo env 25`     | Set up the environment for the Java version given as an argument. Ignore `.jlorc` file.                                                                                                       |
| `jlo init`       | Create a `.jlorc` file that pins the **latest available** Java version.                                                                                                                       |
| `jlo init 25`    | Create a `.jlorc` file that pins the given Java version. Ignore `.jlorc` file.                                                                                                                |
| `jlo update`     | Update the Java version from the `.jlorc` file to the latest minor release.                                                                                                                   |
| `jlo update 25`  | Update the specified Java version to the latest minor version. Ignore `.jlorc` file.<br>Multiple versions can be specified, e.g. `jlo update 8 11 17`.<br>Missing versions will be installed. |
| `jlo update all` | Update all installed Java versions to their latest minor releases. Ignore `.jlorc` file.                                                                                                      |
| `jlo clean`      | Keep only the latest minor version of each installed major version, remove all others.                                                                                                        |
| `jlo selfupdate` | Update J'Lo itself to the latest version.                                                                                                                                                     |
| `jlo version`    | Print the currently installed J'Lo version.                                                                                                                                                   |

## Uninstalling J'Lo

To uninstall J'Lo, simply remove the `~/.jlo/` directory and the lines you added to your shell profile during
installation.

You may also want to remove the `~/.jdks/` directory (or `~/Library/Java/JavaVirtualMachines/` on macOS) if you no
longer need the installed JDKs.
