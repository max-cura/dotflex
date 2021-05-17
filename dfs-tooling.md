Usage:

    dfs-tooling new [name] -- <...>
    dfs-tooling bind <name> <file>...
    dfs-tooling delete <file>...
    dfs-tooling upsync
    dfs-tooling downsync
    dfs-tooling init <repo>
    dfs-tooling feature [-only-files] <+name|-name>...

Basic process:

- keeps a cloned repo in $HOME/.dfs-tooling/repo/
- centred on 'features'
    - a feature is a set of software, relevant environment/configuration files, and relevant install/uninstall scripts

For example, 'zsh' is a feature; `dfs-tooling feature +zsh` would do the following:
```
    check whether zsh is installed or not
        if not, run REPO/features/zsh/install[.*]
        if yes, then note that down in LOCAL/active-features.toml
    perform an installation according to REPO/features/zsh/manifest.dfs
```
Uninstalling would do the opposite (`dfs-tooling feature -zsh`)
```
    Undo anything done by REPO/features/zsh/manifest.dfs
    if LOCAL/active-features.toml specifies that *we* installed zsh
        then run REPO/features/zsh/uninstall[.*]
```

We can add files to features (in features/FEATURE/manifest.toml) with `bind`
If we want to remove a file (universally), we use `delete`
If we want to upload our local variant, we use `upsync`
iIf we want to download an upstream variant, we use `downsync`
If we want to invoke a simple command declared by a feature, we use `@{FEATURE}::<name>`, e.g. @zsh::update-installed-paths
When we first start off on a system, then we use `init`.

We do not provide support for merges or conflict resolution; feature support code will be designed as much as possible to allow for local customizations without affecting any tracked files.
Additionally, we provide support for `tags` as ways of marking platform inconsistencies.
For example, on MacOS, I might use the following:

    > dfs-tooling init https://github.com/that-cura-kid/_dfs.git
    Cloning repository . . . successful.
    Verifying manifest . . . successful.
    Checking hashes    . . . successful.
    > dfs-tooling feature -verbose -only-files +zsh @MACOS
    Using: /Users/mcura/.dfs/REPO/features/zsh
    Checking for local changes . . . not found.
    Tool is already installed at /usr/local/zsh, version is acceptable.
    Checking for manifest conflicts . . .
        CONFLICT: ~/.zshrc already exists. Keep? (y/n) n
    Checking for manifest conflicts . . . successful.
    Looking for tag @MACOS in manifest . . . found.
    Checking for tag conflicts . . . not found.
    Installing from manifest:
        instantiated ~/.zshrc -> ~/.zshrc.common
        zsh/zshrc-common -> ~/.zshrc.common
        created ~/.zshrc.local
        [ ... snip ... ]
    Installing from tag @MACOS in manifest:
        zsh/zshrc-macos -> ~/.zshrc-macos (-> ~/.zshrc.local)
    Done.

Other tags might be based on number of screens, processor type, or various other technical considerations.

Program structure:

    Manifest
    TrackedFile     - represents an installed file
    Operation       - single task from a manifest
    OperationGraph  - simple task graph formation
    FileHash        - Fast Eq for files
    Feature
    Local           - local metadata
    Repo            - encapsulates functionality related to the repository
    Export          - export a manifest in makefile format

File structures:

    .dfs-tooling/
        REPO/
            features/
                {FEATURE}/
                    install.sh
                    uninstall.sh
                    manifest.dfs
                    ...
            tracked-files.toml
        LOCAL/
            local-decls.dfs
            active-features.toml
            tracked-files.toml
            repo-upstream.toml          -- tracks upstream repo info
            intermediates/              -- installation artefacts
                {FEATURE}/
                    ...

Other notes:
- need to distinguish between generated and copied/moved files, especially with regards to bind'ing
- generated files will be remembered, and not automatically bound

Okay, manifest is NOT toml:

```
!section "install"

!move @/files/zshrc-common %/.zshrc.common
!cmd[generates(~/.zshrc)]
    'echo -e "#!/bin/zsh\nsource $HOME/.zshrc.common\nsource $HOME/.zshrc.local\n" > $HOME/.zshrc'

@macos!cmd[touches(~/.zshrc.local)]
    'echo -e "source $HOME/.zshrc.macos" >> $HOME/.zshrc.local'
@macos!move @/files/scripts/brew-get-keg-only.php @LOCAL/intermediates/zsh/brew-get-keg-only.php

-- creates a command dfs-tooling @zsh::update-brew-paths
@macos!cmd[generates(~/.zshrc.brew-paths)]
    'echo -e "#!/bin/zsh\n" > $HOME/.zshrc.brew-paths'
@macos!decl[feature(zsh)] "update-brew-paths" {
    keg=$(brew info --installed --json=v1
        | php $HOME/.dfs-tooling/LOCAL/intermediates/zsh/brew-get-keg-only.php)
    keg_path="\$PATH"
    for item in $keg ; do
        keg_path="$(brew --prefix $item)/bin:${keg_path}" ; done
    keg_path="/usr/local/bin:$keg_path"
    printf "#!/bin/zsh\n%s" $keg_path > $HOME/.zshrc.brew-paths
}
```

Syntax rules (id lists are incomplete):

```regex
comment := '-' '-' [^\n]* '\n'
tag := '@' identifier
directive := ['cmd' | 'decl' | 'move' | 'section']
string := '{' [^\{\}]* '}' | '\'' [^\']* '\'' | [^\s!@\{\}\']+
number := [0-9]+
parameter := string | number
attr-name := ['generates' | 'touches' | 'feature']
attr-list := attr-name '(' parameter (',' parameter)* ')'
attrs-block := '[' attr-list ']'
statement := tag? '!' directive attrs-block? parameter*
```
