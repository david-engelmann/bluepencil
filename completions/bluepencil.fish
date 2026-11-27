# fish completion for bluepencil
complete -c bluepencil -n '__fish_use_subcommand' -a 'propose' -d 'bluepencil propose'
complete -c bluepencil -n '__fish_use_subcommand' -a 'diff' -d 'bluepencil diff'
complete -c bluepencil -s h -l help -d 'Show help'
complete -c bluepencil -n '__fish_seen_subcommand_from propose' -s 'o' -r -F
complete -c bluepencil -n '__fish_seen_subcommand_from propose' -l 'output' -r -F
complete -c bluepencil -n '__fish_seen_subcommand_from diff' -s 'o' -r -F
complete -c bluepencil -n '__fish_seen_subcommand_from diff' -l 'output' -r -F
