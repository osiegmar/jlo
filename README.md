# J'Lo – the Java Loader

The Java Loader (or J'Lo for short) is a minimalistic tool to download and manage Java installations on your machine.
It is written in Rust and the main focus is on simplicity and ease of use.

J'Lo currently supports Linux (x86_64) and macOS (arm64).

At the moment, only the Eclipse Temurin distribution is available.
Java versions are supported starting from Java 8, with all newer versions working automatically.

## Installing J'Lo

Installing J'Lo on Unix-like systems (Linux, macOS, WSL, etc.):

```shell
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/java-loader/jlo/refs/heads/main/install.sh)"
```

This will download the latest J'Lo binary and install it to `~/.jlo/`. You can safely re-run this command to update
J'Lo to the latest version. This is basically the same as running `jlo selfupdate`.

> [!IMPORTANT]
> Watch the output closely, as you will have to add some lines to your shell profile to make `jlo` available in your
> terminal.

## Quick Start

Setup environment for Java 25 (installing it first, if necessary):

```shell
jlo env 25

# Verify that Java 25 is now active (not needed, only for demonstration purposes)
echo $JAVA_HOME
which java
```

JDKs are installed to `~/.jdks/` on Linux and `~/Library/Java/JavaVirtualMachines/` on macOS.
This allows automatic discovery of installed JDKs by IDEs like IntelliJ IDEA.

## More Commands

| Command       | Description                                                                                        |
|---------------|----------------------------------------------------------------------------------------------------|
| `jlo env`     | Setup environment for the Java version specified in the `.jlorc` file of the current directory.    |
| `jlo init 25` | Creating a `.jlorc` file in the current directory that pins the given Java version.                |
| `jlo init`    | Creating a `.jlorc` file in the current directory that pins the **latest available** Java version. |

> [!TIP]
> If you enabled the J'Lo autoload feature during installation, J'Lo will automatically set up the Java environment
> whenever you `cd` into a directory that contains a `.jlorc` file.

## Maintenance Commands

| Command          | Description                                                                            |
|------------------|----------------------------------------------------------------------------------------|
| `jlo update`     | Update the Java version from the `.jlorc` file to the latest minor release.            |
| `jlo update 25`  | Update specified Java version to the latest minor version.                             |
| `jlo clean`      | Keep only the latest minor version of each installed major version, remove all others. |
| `jlo selfupdate` | Update J'Lo itself to the latest version.                                              |
| `jlo version`    | Print the currently installed J'Lo version.                                            |

## Uninstalling J'Lo

To uninstall J'Lo, simply remove the `~/.jlo/` directory and the lines you added to your shell profile during
installation.

You may also want to remove the `~/.jdks/` directory (or `~/Library/Java/JavaVirtualMachines/` on macOS) if you no
longer need the installed JDKs.
