FROM scratch
ARG TARGETOS TARGETARCH
COPY $TARGETOS-$TARGETARCH/dmarc-report-viewer /
EXPOSE 8080
CMD ["./dmarc-report-viewer"]