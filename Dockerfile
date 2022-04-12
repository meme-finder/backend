# Docker - это программная платформа для быстрой
# разработки, тестирования и развертывания приложений

# Каждый этап сборки кешируется (сохраняется на будущее),
# поэтому при каждой сборке проекта не нужно заново
# компилировать зависимости

# Docker собирает программы в "контейнеры"
# и запускает их 

# Скачивание образа для компиляции Rust
FROM rust:1.60 as build

# Создание папки программы
RUN USER=root cargo new --bin backend
WORKDIR /backend

# Копирование файлов с информацией о проекте
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Компиляция зависимостей
RUN cargo build --release && rm ./src/*.rs ./target/release/deps/backend*

# Копирование файлов с кодом
ADD . ./

# Компиляция программы
RUN cargo build --release

# Скачивание образа для запуска программы
FROM debian:11-slim

WORKDIR /app

# Копирование исполняемых файлов
COPY --from=build /backend/target/release/backend /usr/local/bin
COPY ./static /app/static

# Запуск программы
CMD [ "/usr/local/bin/backend" ]
