# PowerShell completion for bluepencil
Register-ArgumentCompleter -Native -CommandName bluepencil -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)
    $candidates = @('propose', 'diff', '-o', '--output', '-h', '--help')
    $candidates | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
    }
}
