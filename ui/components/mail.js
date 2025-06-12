import { LitElement, html, css } from "lit";
import { globalStyle } from "./style.js";

export class Mail extends LitElement {
    static styles = [globalStyle, css`
        .error pre {
            border: 1px solid #e0e0e0;
            border-radius: 3px;
            background-color: #efefef;
            padding: 5px;
        }
    `];

    static get properties() {
        return {
            uid: { type: String },
            mail: { type: Object, attribute: false },
            reports: { type: Object, attribute: false },
            errors: { type: Object, attribute: false }
        };
    }

    constructor() {
        super();
        this.uid = null;
        this.mail = null;
        this.reports = {};
        this.errors = {};
    }

    async updated(changedProperties) {
        if (changedProperties.has("uid") && changedProperties.uid !== this.uid && this.uid) {
            fetch("mails/" + this.uid)
                .then(async (response) => {
                    this.mail = await response.json();
                });
            fetch("dmarc-reports?uid=" + this.uid)
                .then(async (response) => {
                    this.reports.dmarc = await response.json();
                });
            fetch("tlsrpt-reports?uid=" + this.uid)
                .then(async (response) => {
                    this.reports.tlsrpt = await response.json();
                });
            fetch("mails/" + this.uid + "/errors")
                .then(async (response) => {
                    this.errors = await response.json();
                });
        }
    }

    renderOversized(oversized) {
        if (oversized) {
            return html`<span class="badge badge-negative">Yes</span>`;
        } else {
            return html`<span class="faded">No</span>`;
        }
    }

    render() {
        if (!this.mail) {
            return html`No mail loaded`;
        }

        return html`
            <h1>Mail Details</h1>
            <table>
                <tr>
                    <td class="name">UID</td>
                    <td>${this.mail.uid}</td>
                </tr>
                <tr>
                    <td class="name">Size</td>
                    <td>${this.mail.size} Bytes</td>
                </tr>
                <tr>
                    <td class="name">Oversized</td>
                    <td>${this.renderOversized(this.mail.oversized)}</td>
                </tr>
                <tr>
                    <td class="name">Date</td>
                    <td>${new Date(this.mail.date * 1000).toLocaleString()}</td>
                </tr>
                <tr>
                    <td class="name">Subject</td>
                    <td>${this.mail.subject}</td>
                </tr>
                <tr>
                    <td class="name">Sender</td>
                    <td>
                        <a href="#/mails?sender=${encodeURIComponent(this.mail.sender)}">
                            ${this.mail.sender}
                        </a>
                    </td>
                </tr>
                <tr>
                    <td class="name">Recipient</td>
                    <td>${this.mail.to}</td>
                </tr>
            </table>

            ${Object.values(this.reports).every((reports) => reports.length === 0) ?
                html`<p>No reports found.</p>`
                : html``
            }

            ${"dmarc" in this.reports && this.reports.dmarc.length > 0 ?
                html`
                    <h2>DMARC Reports</h2>
                    <drv-dmarc-report-table .reports="${this.reports.dmarc}"></drv-dmarc-report-table>`
                : html``
            }

            ${"xml" in this.errors && this.errors.xml.length > 0 ?
                html`
                    <h2>XML Parsing Errors</h2>
                    ${this.errors.xml.map((e) =>
                    html`
                        <div class="error">
                            ${e.error}
                            <pre>${e.report}</pre>
                        </div>`
                    )}`
                : html``
            }

            ${"tlsrpt" in this.reports && this.reports.tlsrpt.length > 0 ?
                html`
                    <h2>TLS-RPT Reports</h2>
                    <drv-tlsrpt-report-table .reports="${this.reports.tlsrpt}"></drv-tlsrpt-report-table>`
                : html``
            }

            ${"json" in this.errors && this.errors.json.length > 0 ?
                html`
                    <h2>JSON Parsing Errors</h2>
                    ${this.errors.json.map((e) =>
                    html`
                        <div class="error">
                            ${e.error}
                            <pre>${e.report}</pre>
                        </div>`
                    )}`
                : html``
            }
        `;
    }
}

customElements.define("drv-mail", Mail);
