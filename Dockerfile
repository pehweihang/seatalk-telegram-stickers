FROM lukemathwalker/cargo-chef:latest AS chef
WORKDIR /app

FROM chef AS planner
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src ./src
RUN cargo chef prepare

FROM chef AS builder
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release
COPY . .
RUN cargo build --release
RUN mv ./target/release/seatalk-tgs ./app

FROM edasriyan/lottie-to-gif AS lottie-to-gif

FROM debian:stable-slim AS runtime

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get -y update && apt-get -y upgrade && apt-get install -y ffmpeg gifsicle

WORKDIR /app
COPY --from=builder /app/app /usr/local/bin/
COPY ./config/ /app/config
COPY --from=lottie-to-gif /usr/bin/lottie_to_png /usr/bin/lottie_to_png
ENTRYPOINT ["/usr/local/bin/app"]
