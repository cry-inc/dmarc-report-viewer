import { LitElement, html, nothing } from "lit";
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
    }

    prepareIssueBadges(issues) {
        // Sort to always have the same badge order
        issues.sort();

        // Convert to nice bades with tool tips
        return issues.map(issue => {
            if (issue === "SpfPolicy") {
                return html`<span class="badge badge-negative help" title="DMARC SPF Policy Issue">SPF Policy</span> `
            } else if (issue === "SpfAuth") {
                return html`<span class="badge badge-negative help" title="DMARC SPF Auth Issue">SPF Auth</span> `
            } else if (issue === "DkimPolicy") {
                return html`<span class="badge badge-negative help" title="DMARC DKIM Policy Issue">DKIM Policy</span> `
            } else if (issue === "DkimAuth") {
                return html`<span class="badge badge-negative help" title="DMARC DKIM Auth Issue">DKIM Auth</span> `
            } else {
                html`<span class="badge badge-negative">${issue}</span> `
            }
        })
    }

    render() {
        return html`
            <h1>DMARC Mail Sources</h1>
            <div>
                ${this.filtered ? html`Filter active! <a class="ml button" href="#/sources">Show all Sources</a>` : nothing}
            </div>
            <table>
                <tr>
                    <th>IP Address</th>
                    <th class="help" title="Number of records from reports for this IP">Count</th>
                    <th class="sm-hidden">Domain</th>
                    <th class="xs-hidden help" title="Issues detected in reports from this IP">Issues</th>
                </tr>
                ${this.sources.length !== 0 ? this.sources.map((source) =>
                    html`<tr> 
                        <td>${source.ip}</a></td>
                        <td>${source.count}</td>
                        <td class="sm-hidden"><a href="#/sources?domain=${encodeURIComponent(source.domain)}">${source.domain}</a></td>
                        <td class="xs-hidden">${this.prepareIssueBadges(source.issues)}</td>
                    </tr>`
                ) : html`<tr>
                        <td colspan="4">No sources found.</td>
                    </tr>`
            }
            </table>
        `;
    }
}

customElements.define("drv-sources", Sources);
