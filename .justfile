hello:
  echo hi

deploy:
  cargo geng build --platform web --release
  butler push target/geng kuviman/sgj2024:html5
