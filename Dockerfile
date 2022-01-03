FROM rust:latest
WORKDIR /pj
COPY . .
RUN cargo install
ENV PORT 8080
CMD ["cloudrun-1"]