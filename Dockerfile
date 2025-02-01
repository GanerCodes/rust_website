FROM alpine:latest

RUN apk add --no-cache curl build-base
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal --default-toolchain nightly
COPY . /app
WORKDIR /app/src
RUN mv /app/config.rs /app/src/config.rs
RUN . "$HOME/.cargo/env" && rustc main.rs -o ./main
CMD ["./main"]