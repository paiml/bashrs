FROM scratch
COPY rash /rash
USER 65534:65534
ENTRYPOINT ["/rash"]
