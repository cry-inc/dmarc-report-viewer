import { LitElement, html } from "lit";
import { globalStyle } from "../style.js";

export class Sources extends LitElement {
    static styles = [globalStyle];

    static properties = {
        params: { type: Object },
        sources: { type: Array },
    };

    constructor() {
        super();
        this.params = {};
        this.sources = [];
        this.filtered = false;
    }

    updated(changedProperties) {
        if (changedProperties.has("params")) {
            this.updateSources();
        }
    }

    async updateSources() {
        const sourcesResponse = await fetch("sources");
        this.filtered = false;
        this.sources = await sourcesResponse.json();
        if (this.params.domain) {
            this.sources = this.sources.filter(s => s.domain === this.params.domain);
            this.filtered = true;
        }
        if (this.params.issues) {
            this.sources = this.sources.filter(s => s.issues.length > 0);
            this.filtered = true;
        }
        if (this.params.type) {
            this.sources = this.sources.filter(s => s.types.includes(this.params.type));
            this.filtered = true;
        }
    }

    prepareIssueBadges(issues) {
        // Sort to always have the same badge order
        issues.sort();

        // Convert to nice bades with tool tips
        return issues.map(issue => {
            if (issue === "SpfPolicy") {
                return html`<span class="badge badge-negative">SPF Policy</span> `;
            } else if (issue === "SpfAuth") {
                return html`<span class="badge badge-negative">SPF Auth</span> `;
            } else if (issue === "DkimPolicy") {
                return html`<span class="badge badge-negative">DKIM Policy</span> `;
            } else if (issue === "DkimAuth") {
                return html`<span class="badge badge-negative">DKIM Auth</span> `;
            } else if (issue === "StarttlsNotSupported") {
                return html`<span class="badge badge-negative">No STARTTLS Support</span> `;
            } else if (issue === "CertificateHostMismatch") {
                return html`<span class="badge badge-negative">Certificate Mismatch</span> `;
            } else if (issue === "CertificateExpired") {
                return html`<span class="badge badge-negative">Certificate Expired</span> `;
            } else if (issue === "CertificateNotTrusted") {
                return html`<span class="badge badge-negative">No Certificate Trust</span> `;
            } else if (issue === "ValidationFailure") {
                return html`<span class="badge badge-negative">Validation Failure</span> `;
            } else if (issue === "TlsaInvalid") {
                return html`<span class="badge badge-negative">TLSA Invalid</span> `;
            } else if (issue === "DnssecInvalid") {
                return html`<span class="badge badge-negative">DNSSEC Invalid</span> `;
            } else if (issue === "DaneRequired") {
                return html`<span class="badge badge-negative">DANE Required</span> `;
            } else if (issue === "StsPolicyFetchError") {
                return html`<span class="badge badge-negative">STS Policy Fetch Error</span> `;
            } else if (issue === "StsPolicyInvalid") {
                return html`<span class="badge badge-negative">STS Policy Invalid</span> `;
            } else if (issue === "StsWebpkiInvalid") {
                return html`<span class="badge badge-negative">STS WebPKI Invalid</span> `;
            } else {
                return html`<span class="badge badge-negative">${issue}</span> `;
            }
        })
    }

    prepareTypesBadges(source) {
        // Sort to always have the same badge order
        source.types.sort();

        // Convert to nice bades with tool tips
        return source.types.map(type => {
            if (type === "Tls") {
                return html`<a class="button sm help" href="#/tls-reports?ip=${encodeURIComponent(source.ip)}" title="Show all SMTP TLS reports for this IP">SMTP TLS</a> `;
            } else if (type === "Dmarc") {
                return html`<a class="button sm help" href="#/dmarc-reports?ip=${encodeURIComponent(source.ip)}" title="Show all DMARC reports for this IP">DMARC</a> `;
            }
        })
    }

    render() {
        return html`
            <h1>DMARC Mail Sources</h1>
            <div>
                ${this.filtered ?
                    html`Filter active! <a class="ml button" href="#/sources">Show all Sources</a>` :
                    html`Filters: <a class="ml button" href="#/sources?issues=true">Only Sources with Issues</a>
                    <a class="ml button" href="#/sources?type=Dmarc">Only Sources from DMARC Reports</a>
                    <a class="ml button" href="#/sources?type=Tls">Only Sources from SMTP TLS Reports</a>`
                }
            </div>
            <table>
                <tr>
                    <th>IP Address</th>
                    <th class="help" title="Number of records from reports for this IP">Count</th>
                    <th class="sm-hidden">Domain</th>
                    <th class="sm-hidden help" title="Report Types">Types</th>
                    <th class="xs-hidden help" title="Issues detected in reports from this IP">Issues</th>
                </tr>
                ${this.sources.length !== 0 ? this.sources.map((source) =>
                    html`<tr> 
                        <td>${source.ip}</a></td>
                        <td>${source.count}</td>
                        <td class="sm-hidden"><a href="#/sources?domain=${encodeURIComponent(source.domain)}">${source.domain}</a></td>
                        <td class="sm-hidden">${this.prepareTypesBadges(source)}</td>
                        <td class="xs-hidden">${this.prepareIssueBadges(source.issues)}</td>
                    </tr>`
                ) : html`<tr>
                        <td colspan="5">No sources found.</td>
                    </tr>`
            }
            </table>
        `;
    }
}

customElements.define("drv-sources", Sources);
