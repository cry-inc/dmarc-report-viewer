import { LitElement, html, css, nothing } from "lit";
import { globalStyle } from "../style.js";

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
            id: { type: String },
            mail: { type: Object, attribute: false },
            reportsDmarc: { type: Array, attribute: false },
            reportsTls: { type: Array, attribute: false },
            errors: { type: Object, attribute: false }
        };
    }

    constructor() {
        super();
        this.id = null;
        this.mail = null;
        this.reportsDmarc = [];
        this.reportsTls = [];
        this.errors = {};
    }

    async updated(changedProperties) {
        if (changedProperties.has("id") && changedProperties.id !== this.id && this.id) {
            fetch("mails/" + this.id).then(async (response) => {
                this.mail = await response.json();
            });
            fetch("dmarc-reports?id=" + this.id).then(async (response) => {
                this.reportsDmarc = await response.json();
            });
            fetch("tls-reports?id=" + this.id).then(async (response) => {
                this.reportsTls = await response.json();
            });
            fetch("mails/" + this.id + "/errors").then(async (response) => {
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
                    <td class="name">Account</td>
                    <td>${this.mail.account}</td>
                </tr>
                <tr>
                    <td class="name">Folder</td>
                    <td>${this.mail.folder}</td>
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

            ${this.reportsDmarc.length === 0 && this.reportsTls.length === 0 ?
                html`<p>No reports found!</p>` : nothing
            }

            ${this.reportsDmarc.length > 0 ?
                html`
                    <h2>DMARC Reports</h2>
                    <drv-dmarc-report-table .reports="${this.reportsDmarc}"></drv-dmarc-report-table>`
                : nothing
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
                : nothing
            }

            ${this.reportsTls.length > 0 ?
                html`
                    <h2>SMTP TLS Reports</h2>
                    <drv-tls-report-table .reports="${this.reportsTls}"></drv-tls-report-table>`
                : nothing
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
                : nothing
            }
        `;
    }
}

customElements.define("drv-mail", Mail);
