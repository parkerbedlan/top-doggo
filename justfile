tailwind-compile:
    npx tailwindcss -i ./assets/input.css -o ./assets/output.css
tw: tailwind-compile

tailwind-compile-watch:
    npx tailwindcss -i ./assets/input.css -o ./assets/output.css --watch
tww: tailwind-compile-watch

watch:
    cargo watch -x run
w: watch

fix:
    cargo watch -x fix

db:
    sqlite3 todos.db

#tmux new-session -d -s best-doggo \; send-keys 'vim .' Enter \; new-window \; send-keys 'just w' Enter \; new-window \; send-keys 'git pull' Enter \; new-window \; send-keys 'just tww' Enter \; attach-session -t best-doggo
dev:
    tmux new-session -d -s best-doggo \; send-keys 'vim .' Enter \; new-window \; send-keys 'just w' Enter \; new-window \; send-keys 'git pull' Enter \; attach-session -t best-doggo

