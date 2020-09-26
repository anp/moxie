const vscode = require('vscode');

function activate(context) {
	// only executed once when extension is activated
	console.log('honkity honk');
	vscode.window.showInformationMessage('honk starting...');

	// const focusProvider = new FocusProvider(vscode.workspace.rootPath);
	// vscode.window.registerTreeDataProvider('focus', focusProvider);
	// TODO make this a panel thingy

	vscode.window.showInformationMessage('honk started');
}
exports.activate = activate;

function deactivate() { }

module.exports = {
	activate,
	deactivate
}
