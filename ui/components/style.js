import { css } from "lit";

export const globalStyle = css`
    a {
        color: rgb(0, 123, 255);
        text-decoration: none;
    }

    a:hover {
        color: rgb(0, 86, 179);
    }

    .badge {
        border-radius: 3px;
        padding-left: 4px;
        padding-right: 4px;
        background-color: #888;
        color: white;
    }

    .badge-negative {
        background-color: rgb(220, 53, 69);
    }

    .badge-positive {
        background-color: rgb(25, 135, 84);
    }

    .faded {
        color: #ccc;
    }

    .help {
        cursor: help;
        text-decoration-line: underline;
        text-decoration-style: dotted;
    }

    table {
        width: 100%;
        margin-top: 15px;
        border-collapse: collapse;
    }

    th {
        color: #495057;
        background-color: #e9ecef;
        border-bottom: 2px solid #dee2e6;
        text-align: left;
        font-weight: 700;
        font-size: 17px;
    }

    td {
        border-top: 1px solid #dee2e6;
    }

    td, th {
        padding-left: 15px;
        padding-right: 15px;
        padding-top: 5px;
        padding-bottom: 5px;
    }

    tr:hover {
        background-color: #f4f4f4;
    }

    td.name {
        font-weight: 700;
        width: 175px;
        color: rgb(73, 80, 87);
    }

    h1, h2, h3 {
        padding: 0px;
        margin-top: 15px;
        margin-bottom: 15px;
    }

    h1 {
        margin-top: 0px;
    }

    .button {
        background: none;
	    border: none;
	    font: inherit;
	    cursor: pointer;
	    outline: inherit;
        display: inline-block;
        padding: 5px;
        padding-left: 8px;
        padding-right: 8px;
        margin-right: 10px;
        color: white;
        border-radius: 3px;
        background-color: rgb(108, 117, 125);
        margin-bottom: 3px;
    }

    .button:hover {
        color: white;
        background-color: rgb(90, 98, 104);
    }

    .button.sm {
        padding: 1px;
        padding-left: 4px;
        padding-right: 4px;
        margin-right: 5px;
    }

    .ml {
        margin-left: 10px;
    }

    .sourceip .name {
        padding-left: 40px;
    }

    .mr-5 {
        margin-right: 5px;
    }

    @media only screen and (max-width: 1100px) {
        .md-hidden {
            display: none;
        }
    }

    @media only screen and (max-width: 800px) {
        .sm-hidden {
            display: none;
        }
    }
    
    @media only screen and (max-width: 600px) {
        .xs-hidden {
            display: none;
        }
    }
`;
