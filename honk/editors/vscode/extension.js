const vscode = require('vscode');

function activate(context) {
	// only executed once when extension is activated
	console.log('honkity honk');

	// TODO make this a panel thingy
	let disposable = vscode.commands.registerCommand('honk.boot', function () {
		// executed every time command is executed
		vscode.window.showInformationMessage('honk');
	});

	context.subscriptions.push(disposable);
}
exports.activate = activate;

function deactivate() { }

module.exports = {
	activate,
	deactivate
}
