tailwind-compile:
    npx tailwindcss -i ./assets/input.css -o ./assets/output.css
tw: tailwind-compile

tailwind-compile-watch:
    npx tailwindcss -i ./assets/input.css -o ./assets/output.css --watch
tww: tailwind-compile-watch

watch:
    cargo watch -x run
w: watch

dev:
    tmux new-session -d -s dev_session \; send-keys 'just tww' Enter \; new-window \; send-keys 'just w' Enter \; new-window \; send-keys 'git pull' Enter \; attach-session -t dev_session
