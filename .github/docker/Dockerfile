FROM scratch
ARG TARGETOS TARGETARCH
COPY $TARGETOS-$TARGETARCH/dmarc-report-viewer /
EXPOSE 8080
HEALTHCHECK --start-period=30s CMD ["./dmarc-report-viewer", "--health-check"]
CMD ["./dmarc-report-viewer"]
