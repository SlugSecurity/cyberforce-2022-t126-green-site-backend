FROM archlinux:latest AS base
EXPOSE 8080
COPY ./target/release/green-site-backend /
CMD ["/green-site-backend"]