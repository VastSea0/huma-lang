const path = require("path");

const vscode = require("vscode");
const {
  LanguageClient,
  TransportKind,
} = require("vscode-languageclient/node");

/** @type {LanguageClient | undefined} */
let client;

function activate(context) {
  const cfg = vscode.workspace.getConfiguration("huma");
  const serverPath = cfg.get("lsp.serverPath", "huma-lsp");

  const serverOptions = {
    command: serverPath,
    args: [],
    transport: TransportKind.stdio,
  };

  const clientOptions = {
    documentSelector: [{ scheme: "file", language: "huma" }],
  };

  client = new LanguageClient(
    "huma-lsp",
    "Hüma Language Server",
    serverOptions,
    clientOptions
  );

  context.subscriptions.push(client.start());
}

async function deactivate() {
  if (!client) return;
  await client.stop();
}

module.exports = { activate, deactivate };

