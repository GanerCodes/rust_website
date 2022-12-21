FROM alpine:latest

RUN apk add --no-cache curl build-base

COPY . /app
WORKDIR /app/src

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal --default-toolchain nightly
RUN . "$HOME/.cargo/env" && rustc main.rs -o ./main

CMD ["./main"]