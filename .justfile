hello:
  echo hi

deploy:
  cargo geng build --platform web --release
  butler push target/geng kuviman/meowjelly:html5
