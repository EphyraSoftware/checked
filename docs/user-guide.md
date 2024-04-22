## User guide

Checked comes in two parts, the app that runs on Holochain (which comes with a UI) and the command line tool that you 
use to fetch and sign assets. This guide will help you get both installed and walk you through your first asset fetch.

### Pre-requisite: Holochain launcher

You can run Holochain any way you like, but the recommended way to get started is to use the Holochain launcher. You
find an appropriate launcher release on its [releases page](https://github.com/holochain/launcher/releases).

Each release of `checked` is built against a specific version of Holochain. Holochain apps can sometimes run on newer 
versions of Holochain but it's not guaranteed. Check the release notes for the version of `checked` that you are 
going to use to see which version of Holochain is recommended.

### Installing the app

Find the latest release on the [releases page](https://github.com/holochain/launcher/releases) and download 
the `.webhapp` file.

Install the `.webhapp` in the Holochain launcher by installing from the file system and selecting the `.webhapp` file.

Once it is installed, you can launch the app by clicking on it. You will need to create a signing key and fetch an 
asset before there's much to look at!

### Installing the command line tool

Back on the [releases page](https://github.com/holochain/launcher/releases) you will find several `checked` files. You
need to choose the one that matches your platform and download it. For example, if you are on Linux you would download
the `checked-linux` file.

Once you have downloaded the file, you need to make sure it is executable and in your path.

Once you have done this, you should be able to run `checked --version` on Linux and MacOS, or `checked.exe --version`
on Windows.

### Fetching an asset

Now you have both pieces installed, you can set up your environment and fetch an asset. Start by creating a signing key.

This can be done with the following command:

```bash
checked generate
```

You will be prompted for a password. This password is used to encrypt your private signing key. You will need to enter 
this password each time you use the private signing key. There is no way to recover a forgotten password so make sure
to pick something you can remember, or record somewhere secure.

Once you have provided a password, you will be prompted to distribute the key on Holochain. Yes `y` to approve this 
action. 

At this point you may be prompted to pick a Holochain process. This happens when you have multiple Holochain processes
running. You will need to figure out which Holochain instance is which in this case. If you just have one Holochain
instance running then the tool will find it for you and you can ignore this paragraph!

If the tool was able to distribute your key on Holochain you should see `Successfully distributed on Holochain!` at the
end of the output.

Now you can fetch an asset. You can fetch any asset you'd like at this point. However, you'll get a better idea of what
it looks like to fetch an asset that other people have signed if you fetch the recommended asset:

```bash
checked fetch https://github.com/EphyraSoftware/checked/releases/download/v0.1.0/user-guide.txt
```

Again, you may be prompted to pick a Holochain instance. If this is the case and you now know which one to pick, you
can check the output from the previous command and look for `Connecting to admin port <port-number>`. This is the admin
port for the Holochain instance you're working with. In future you can pass `--port <port-number>` to `checked` 
commands that will need to connect to this Holochain instance. That will save you from having to pick the instance each
time you run a command.

The download should succeed and you should see the message `Downloaded to <temporary-path>`. You will now get a prompt
to create your own signature for this asset. Whether you do this in the future is up to you, but for now, press `y`
to approve this prompt. You will be prompted for your password. This is the password you entered when you created your
private signing key.

If the tool was able to publish your signature on Holochain you should see `Created signature!` at the end of the 
output.

It is important to know that when you create a signature, you are publishing this to other users for them to check 
against. By publishing your signature you are contributing to the security of the asset. However, you are also creating
a public record that you downloaded it. You should be aware of this when you are fetching assets and make you own
decision about whether to publish a signature for each asset you fetch.

That's it! You're up and running. You can continue using the `checked fetch` command to fetch assets and check their
signatures. You will run into assets that have no signatures, feel free to be the first one to create a signature for
them!

If you run into any issues or have questions, then please feel free to open an issue on the 
[GitHub repository](https://github.com/EphyraSoftware/checked/issues/new).
