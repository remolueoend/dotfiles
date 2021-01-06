# dotfiles

**⚠ This project is under development and not yet ready for use in production ⚠**

This app provides a simple dotfiles manager. It only manages the linking of configuration files from your dotfiles directory into your home directory. It leaves it up to you how you want to store and version you dotfiles, be it a GIT repository or a Google-Drive folder.

There are many dotfiles managers out there and even more contradicting philosophies how dotfiles should be managed in general. This project sticks to the following ideas, but keep in mind: There is no right or wrong, these ideas are fully subjective to the author of this tool:
1. Dotfiles are kept in a separate directory. The user home directory should not be a GIT repository root.
2. Every user has their personal workflow how to store and manage configuration files. This tool should not interfere with it.
3. It should be explicitly declared which files are part of the dotfiles repository to avoid publishing any kind of sensitive information


The goals of this project are:
1. Keep it as simple as possible to add and remove files
2. Provide a detailed overview of which files are currently linked, unlinked or missing
3. Make it as hard as possible to link unwanted files (such as secrets) and push them to the public
4. Use sane defaults but let users customize the tool to their liking.


## Install
Up to now, there is no installer available. Clone this repo and build it from source. This tool is written in Rust and requires the [Rust toolchain](https://www.rust-lang.org/tools/install) to build it locally.

## Usage
You can use `dotfiles -h` or `dotfiles <COMMAND> -h` to get a detailed description of the interface.

The following chapters describe the different commands in more detail.

### Configuration
A human-readable configuration file is used to provide a list of all configuration files from your dotfiles directory which should be linked to your home directory. Commands such as `add` and `remove` help you to update the list of files to link. The `status` command gives you an overview of your linked files. All commands which lead to changes in your configuration or file system provide a `--dry` flag allowing you to see what would happen when a command is executed.

### STATUS Command
Prints a detailed list of all files listed under `mappings` in the configuration file.