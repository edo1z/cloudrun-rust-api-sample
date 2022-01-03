FROM rust:latest
WORKDIR /pj
COPY . .
RUN cargo build --release
ENV PORT 8080
EXPOSE 8080
ENTRYPOINT [ "target/release/cloudrun-1" ]