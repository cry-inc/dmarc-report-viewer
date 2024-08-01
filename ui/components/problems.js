import { LitElement, html, css } from "lit";

export class Problems extends LitElement {
    static styles = css`
        h1 {
            font-size: 20px;
        }

        pre {
            border: 1px solid #e0e0e0;
            border-radius: 3px;
            background-color: #efefef;
            padding: 5px;
        }

        .err {
            margin-bottom: 50px;
        }
    `;

    static properties = {
        xmlErrors: { type: Array },
    };

    constructor() {
        super();
        this.xmlErrors = [];
        this.updateProblems();
    }

    async updateProblems() {
        const response = await fetch("xml-errors");
        this.xmlErrors = await response.json();
    }

    render() {
        return html`
            <h1>XML Parsing Errors</h1>
            ${this.xmlErrors.length == 0 ? html`No errors found.` : html``}
            ${this.xmlErrors.map((e) =>
            html`
                <div class="err">
                    ${e.error}
                    <pre>${e.xml}</pre>
                </div>`
            )}
        `;
    }
}

customElements.define("dmarc-problems", Problems);
