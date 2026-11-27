# bash completion for bluepencil
_bluepencil() {
    local cur prev sub sub_opts line
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    if [[ ${COMP_CWORD} -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "propose diff -h --help" -- "${cur}") )
        return 0
    fi
    sub="${COMP_WORDS[1]}"
    sub_opts="-h --help"
    case "${sub}" in
        propose) sub_opts="-o --output -h --help" ;;
        diff) sub_opts="-o --output -h --help" ;;
    esac
    case "${cur}" in
        -*)
            COMPREPLY=( $(compgen -W "${sub_opts}" -- "${cur}") )
            return 0
            ;;
    esac
    compopt -o filenames 2>/dev/null || true
    while IFS= read -r line; do
        COMPREPLY+=( "${line}" )
    done < <(compgen -f -- "${cur}")
}
complete -F _bluepencil bluepencil
