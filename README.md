# dotflex

Dotflex is a dotfile synchronization tool!  
Still in development & pretty brittle, so don't use it unless you're sure it's safe (spoiler alert: we're not willing to use these on our actual $HOME dotfiles, we've only done sandboxed testing, so you probably shouldn't be using it yet either).

### The model

Dotfiles are organized by 'feature'-a feature encompasses some set of dotfiles that should be grouped together for purposes of installation.
For instance, one might have a 'zsh' feature, a 'zsh-linux' feature, and a 'zsh-macos' feature. The 'zsh' feature would be applicable on all my machines, and then 'zsh-linux' or 'zsh-macos' would be layered on top of the base 'zsh' feature on the appropriate machines.

dotflex is further organized around three locations:
- the "target" dotfiles directory (i.e. $HOME)
- the local "repo" dotfiles directory (defaults to $HOME/.dotflex/REPO)
- the remote repository (github.com/{USER}/my-dotfiles-or-sth.git)

Dotflex allows two-way synchronization between the local repo and the remote repository with the `upsync` and `downsync` commands.

Synchronization between the local repo and the target directory is more complex: files from the local repo can be copied to the target directory with `dotflex feature -e [FEATURE_NAME]`, and files in the target directory can be copied to the local repo with `dotflex bind <FEATURE_NAME> -f file`, and, once 'bound', files can be re-copied with `dotflex rebind <FEATURE_NAME> <file>...`.

This system is designed to allow for local modifications without actually affecting the upstream repositories.

There are also mechanisms in place that allow for creating features with more nuanced installation procedures through manually editing files in the local repo (specifically `features/{FEATURE_NAME}/manifest.yml`).
In addition to simply copying files, dotflex supports appending files to files, and the running of arbitrary shell strings or executables.

Documentation is VERY incomplete, and the API is most certainly not stable at this point, so details on that coming later (the mechanisms are in there and are functional--you can check out `src/dotflex/operations.rs` if you're curious--but compared to the file copying mechanisms, the features aren't fully complete yet).

### Environmental variables

- `DOTFLEX_CONFIG_PATH`, defaults to `$HOME/.dotflex`
- `DOTFLEX_TARGET_PATH`, defaults to `$HOME`
