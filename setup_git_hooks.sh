#!/bin/sh

Hooks_dir=".git/hooks"
Pre_commit_file=$Hooks_dir/pre-commit
Pre_commit_hook="\
#!/bin/sh\n\
if ! cargo fmt -- --check > /dev/null 2>&1; then\n\
    echo \"Please run 'cargo fmt' to format your code before making a commit.\"\n\
    exit 1\n\
fi\n\
"

if [ ! -d "$Hooks_dir" ]; then
    echo "The '.git/hooks' directory does not exist. Are you in the root of a git repository?"
    exit 1
fi

rm -f $Pre_commit_file
echo $Pre_commit_hook > $Hooks_dir/pre-commit
chmod +x $Pre_commit_file

echo "Git hooks setup successfully!"