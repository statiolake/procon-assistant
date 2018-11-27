"use strict";

function initializeScript()
{
    return [new host.apiVersionSupport(1, 3)];
}

function invokeScript()
{
    //
    // Insert your script content here.  This method will be called whenever the script is
    // invoked from a client.
    //
    // See the following for more details:
    //
    //     https://aka.ms/JsDbgExt
    //
    var ctl = host.namespace.Debugger.Utility.Control; 
    ctl.ExecuteCommand("bp main!main");
    ctl.ExecuteCommand("g");
}
