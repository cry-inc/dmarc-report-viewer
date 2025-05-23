import { LitElement, html, css } from "lit";
import { globalStyle } from "./style.js";

export class MailTable extends LitElement {
    static styles = [globalStyle];

    static properties = {
        mails: { type: Array },
    };

    constructor() {
        super();
        this.mails = [];
    }

    prepareSubject(subject) {
        subject = subject.replace(/Report Domain: |Report domain: /, "D: ");
        subject = subject.replace(/Submitter: /, "S: ");
        subject = subject.replace(/Report-ID: /, "ID: ");

        const limit = 70;
        if (subject.length <= limit) {
            return subject;
        } else {
            return subject.substring(0, limit) + "...";
        }
    }

    prepareSize(mail) {
        if (mail.oversized) {
            return html`<span class="badge badge-negative">${mail.size}</span>`;
        } else {
            return mail.size;
        }
    }

    prepareXmlFileCount(mail) {
        if (mail.oversized) {
            return html`<span class="faded">n/a</span>`;
        } else if (mail.xml_files < 1) {
            return html`<span class="badge badge-negative">${mail.xml_files}</span>`;
        } else {
            return html`<span class="faded">${mail.xml_files}</span>`;
        }
    }

    prepareParsingError(mail) {
        if (mail.oversized) {
            return html`<span class="faded">n/a</span>`;
        } else if (mail.parsing_errors > 0) {
            return html`<span class="badge badge-negative">Yes</span>`;
        } else {
            return html`<span class="faded">No</span>`;
        }
    }

    render() {
        return html`
            <table>
                <tr>
                    <th>Subject</th>
                    <th class="sm-hidden">Sender</th>
                    <th class="md-hidden">Date</th>
                    <th class="xs-hidden help" title="Size of E-Mail in Bytes">Size</th>
                    <th class="md-hidden help" title="Number of XML files in the Mail">XMLs</th>
                    <th class="xs-hidden help" title="Did the Mail cause DMARC Parsing Errors?">Errors</th>
                </tr>
                ${this.mails.length !== 0 ? this.mails.map((mail) =>
                    html`<tr> 
                        <td><a href="#/mails/${mail.uid}">${this.prepareSubject(mail.subject)}</a></td>    
                        <td class="sm-hidden"><a href="#/mails?sender=${encodeURIComponent(mail.sender)}">${mail.sender}</a></td>
                        <td class="md-hidden">${new Date(mail.date * 1000).toLocaleString()}</td>
                        <td class="xs-hidden">${this.prepareSize(mail)}</td>
                        <td class="md-hidden">${this.prepareXmlFileCount(mail)}</td>
                        <td class="xs-hidden">${this.prepareParsingError(mail)}</td>
                    </tr>`
                ) : html`<tr>
                        <td colspan="4">No mails found.</td>
                    </tr>`
                }
            </table>
        `;
    }
}

customElements.define("drv-mail-table", MailTable);
