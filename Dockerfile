# Stage 1: Build the application
FROM rust:latest as builder
WORKDIR /usr/src/ordinator
COPY . .
RUN cargo install --path .

EXPOSE 8001

# Stage 2: Create the runtime image
FROM debian:bookworm-slim
COPY --from=builder /usr/local/cargo/bin/ordinator /usr/local/bin/ordinator
COPY test_data/export_test.XLSX /usr/src/ordinator/test_data/export_test.XLSX
COPY parameters/work_order_weight_parameters.json /usr/src/ordinator/parameters/work_order_weight_parameters.json
COPY parameters/work_order_weight_parameters.json /usr/src/ordinator/parameters/work_order_weight_parameters.json

CMD ["ordinator"]