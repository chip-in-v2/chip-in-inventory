FROM scratch
COPY release-assets/chip-in-inventory /chip-in-inventory
ENTRYPOINT ["/chip-in-inventory"]