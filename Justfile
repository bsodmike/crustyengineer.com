dev:
    trap 'kill -9 $twpid | true; exit' EXIT
    ./tailwindcss -i src-styles/main.scss -o static/style.css --watch=always \
        & twpid=$!
    echo "Tailwind: watching"

    zola serve

build:
    ./tailwindcss -i src-styles/main.scss -o static/style.css --minify