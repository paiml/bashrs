import * as path from "path";
import { workspace, ExtensionContext } from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

export function activate(context: ExtensionContext): void {
  const config = workspace.getConfiguration("bashrs");
  const serverPath = config.get<string>("serverPath", "bashrs");

  const serverOptions: ServerOptions = {
    command: serverPath,
    args: ["lsp"],
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [
      { scheme: "file", language: "shellscript" },
      { scheme: "file", language: "makefile" },
      { scheme: "file", language: "dockerfile" },
      { scheme: "file", pattern: "**/*.sh" },
      { scheme: "file", pattern: "**/*.bash" },
      { scheme: "file", pattern: "**/Makefile" },
      { scheme: "file", pattern: "**/Makefile.*" },
      { scheme: "file", pattern: "**/*.mk" },
      { scheme: "file", pattern: "**/Dockerfile" },
      { scheme: "file", pattern: "**/Dockerfile.*" },
    ],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher(
        "**/{*.sh,*.bash,Makefile,Makefile.*,*.mk,Dockerfile,Dockerfile.*}"
      ),
    },
  };

  client = new LanguageClient(
    "bashrs",
    "bashrs Language Server",
    serverOptions,
    clientOptions
  );

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
