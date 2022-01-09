# Tribble

**TL;DR** Tribble is a command-line tool that generates a miniature website from a single configuration file to guide users through contributing to open-source projects.

## The Problem

Imagine you're a user who's new to a big project, and you'd like to add a page to their documentation. *But*, they use a custom documentation system that's ironically not documented anywhere, the `CONTRIBUTING.md` file just says to clone the repo and use the provided build scripts, but they aren't documented either! Usually, most users would give up by now. Some might open an issue on the repository or ask in some other channel, and then they might be guided through the process. But that doesn't help the next user, or the one after them. And if you keep adding to a `CONTRIBUTING.md` file, it tends to become more of a series of blocks of random facts about quirks than a useful resource. Enter Tribble.

As maintainers, we want to focus on *writing code*, not writing documentation, and certainly not writing documentation about how to write more documentation! In large repositories, there are a myriad of ways people can contribute, but a traditional `CONTRIBUTING.md` file just can't encompass them all, and you're likely to end up spending more time helping people through contributing than you are actually solving problems if the project gets big enough.

As users, contributing to a big project can be intimidating, and we want to make sure we do the right thing. But if the documentation is incomplete, or worse nonexistent, that can be very hard! With mammoth projects, a good number of quality contributors are probably lost in the early stage of figuring out how to contribute.

## The Solution

What if every project had a massive flowchart that covered all the ways of contributing to that project, with information about the quirks of each? What if maintainers could write a single configuration file to express this that contained all the information people needed to contribute to their project? And what if that flowchart were interactive, with buttons and text inputs that guide a user through the contributing process? And what if this could work for issues too, with new users being guided through providing all the information necessary for reporting a bug? What if that process could generate labels for an issue that could be applied automatically, effectively providing automatic issue triaging?

This is Tribble.

Tribble is a command-line tool that takes in a configuration file written in [YAML](https://yaml.org) and turns it into a fully-fledged website that has an interactive flowchart for guiding users through the contributing process. At every stage, *tags* can be accumulated, which can be automatically added to GitHub issues reported through Tribble. It's designed to make writing and maintaining contributing documentation easy, to guide users through providing the information necessary for maintainers to triage an issue, and to perform basic triaging (assigning users and adding labels) automatically.

## Installation

Tribble is an entirely self-contained binary built with [Rust](https://rust-lang.org) for performance and safety, which you can download from [the release page](https://github.com/arctic-hen7/tribble/releases) or by using Cargo, Rust's package manager.

### Manual download

If you don't use Rust, the easiest way to get Tribble is to download the latest binary for your system from [the releases page](https://github.com/arctic-hen7/tribble), and then to put that wherever you put your binaries (e.g. `/usr/local/bin` on Linux).

Tribble automatically generates binaries for Windows, MacOS, Linux, and Linux musl (for Alpine Linux Docker containers in particular). If you need a bianry for another platform, you'll need to compile it manually with Cargo, which you can install by following the instructions [here](https://rust-lang.org/tools/install). Then follow the instructions for installing Tribble with Cargo and you'll be all set! (Also, if you think we should have binaries for your platform, feel free to [open an issue](https://github.com/arctic-hen7/tribble/issues/new/choose)).

### With Cargo

If you have Cargo installed, just run the following command and you'll be set!

``` shell
cargo install tribble
```

### Other package managers

In future, Tribble will be distributed on more package managers to make installing it even easier.

## Usage

The Tribble CLI takes in a configuration file, which it expects to be at the root of your project named `tribble.yml`. If you need to name it something else, go ahead, and you can tell the CLI about it by passing through `-c <your-config-file-name-here>`. Regardless of what you name it, it still has to be written in [YAML](https://yaml.org). To start off, put this in `tribble.yml` in a directory.

``` yaml
workflows:
  test:
    sections:
      Start:
        - "Hello, and welcome to the first stage of your contribution journey with Tribble!"
        - { text: "I want to report a bug", link: "Report Bug", tags: [ "C:bug" ] }
        - { text: "I want to suggest an enhancement", link: "Request Enhancement", tags: [ "C:enhancement" ] }
        - { text: "I'd like to add something to the docs", link: "endpoint:Documentation", tags: [ "C:docs" ] }
      Report Bug:
        - "Welcome to the section for reporting bugs!"
        - { id: "bug_description", label: "Describe the bug", type: "text" }
        - { id: "test_textarea", label: "Textarea", type: "multiline" }
        - { id: "test_datetime", label: "Datetime", type: "datetime-local" }
        - { id: "bool", label: "Boolean", type: "boolean" }
        - { text: "This bug occurs on the frontend", link: "endpoint:Bug", tags: [ "A:frontend" ] }
      Request Enhancement:
        - "Welcome to the section for suggesting enhancements!"
        - { id: "feature_area", label: "Which area of the system does your feature affect?", options: [ "Frontend", { text: "Backend", tags: [ "A:backend" ] } ], can_select_multiple: true }
        - { id: "test_select", label: "Which of the following best describes you?", options: [ "User", "Developer" ] }
        - { text: "Continue", link: "endpoint:Enhancement", tags: [] }
    index: Start
    endpoints:
      Bug:
        preamble: "Thank you very much for reporting this bug, we'll get on it right away!"
        text: "This report is reporting a bug. Description: ${bug_description}. Boolean: ${bool}"
        dest_text: "Report on GitHub"
        dest_url: "#"
      Enhancement:
        preamble: "Thanks for the request, we'll take a look!"
        text: "This report is requesting an enhancement to the ${feature_area}."
        dest_text: "Report on GitHub"
        dest_url: "#"
      Documentation: "You can contribute to the docs by doing foobar!"
```

<details>
<summary>How does that work?<summary/>

This is the structure of a Tribble configuration file, which has one top-level property `workflows`. In this example, we define a workflow called `test` (which will be available at `/workflows/test` on the generated website) with several sections. We mark the section called *Start* as the `index`, so the workflow will start there. Then, we define a simple paragraph to be displayed, and then two buttons for navigating to different sections based on the user's input. Notice that, depending on which button the user presses, they'll accumulate different tags (e.g. selecting *Report Bug* would add the `C:bug` tag). In the *Report Bug* section, we define some inputs, a `text` input for a brief bug description (you'd probably use a `multline` in real life), a `multiline`, a `datetime-local`, and a `boolean`. Then, we create another button, which has `endpoint:` at the beginning of its link, which means it'll be sent to an endpoint, specifically the *Bug* endpoint. If we look at that under `endpoints:`, we'll see that it has a preamble paragraph and then some text, which the user will be able to copy into something like a GitHub issue. We then define the properties of a button that will send the user to the place where they can report the issue. Notice that the `text` field here uses `${...}` interpolation syntax, which allows it to reference the values of any inputs in the rest of the workflow. Here, we interpolate the bug description that the user gave. If an input isn't marked as `optional: true`, you can be certain that it will be present for interpolation here. The only other thing particularly of note is the *Documentation* endpoint, which isn't a reporting endpoint like *Bug*, but it instead just provides instructions. These instructional endpoints are particularly useful for walking a user through creating different types of pull requests to your project.

</details>

With that file ready, now run `tribble serve` in the same directory, which will immediately (well, 37 milliseconds by our tests) generate a website for your app, which you can see at <http://localhost:8080>. If you want to change the host or port of that, you can run the command with the `--host` and `--port` flags as appropriate.

And that's how Tribble works!

API docs TODO.

### Internationalization

Everything we just did is all very well, but large projects often have to have documentation in many languages, so how does Tribble handle that? Well, if you create a Tribble file containing this at `tribble.yml`, you'll see!

``` yaml
languages:
  en-US: en-US.yml
```

This defines the languages we'll support and links to configuration files for them. Then, a workflow called `test` in `en-US.yml` will be available at `/workflow/en-US/test`. You can name your locales however you want, but we recommend the `[language]-[REGION]` approach (e.g. `en-US`, `en-GB`, `zh-CN`, `ru-RU`). With that, your Tribble instance now has full support for as many languages as you want!

### CLI commands

The Tribble CLI supports just five commands, but they can be used to create complex and intuitive user experiences for contributors. Note that the location of your configuration file can be changed with the top-level `-c`/`--config` flag (e.g. `tribble -c test.yml serve`).

- `build` -- builds your workflows (called by `tribble serve` automatically, so you shouldn't need this unless you want to investigate the underlying files)
- `clean` -- purges Tribble metadata in the event of a corruption
- `deploy` -- builds your workflows to static files for deployment, generating a `pkg/` folder (changeable with `-o`/`--output`)
- `help` -- displays a help page for the CLI that will tell you everything in this section
- `serve` -- serves your workflows locally for development, watching your Tribble configuration files for changes (you'll need to re-run if you add new locales though)

### Deploying

Oncee you've built some Tribble workflows and you want to deploy them to your website, run `tribble deploy --path <serve-path>` (where `<serve-path>` is the URl of the relative path at which you'll serve Tribble, e.g. `/tribble`) to generate a `pkg/` folder. That will contain static files that you can deploy to any hosting provider that supports serving static assets. For example, on GitHub Pages, you'd just add that folder to the root of your site and rename it to `tribble`, and then you'd be able to access the `test` workflow at `https://<your-username>.github.io/<your-repo>/tribble/workflow/test`. The generated files are fully production-ready, and they'll produce an extremely performant site built with [Perseus](), which uses Rust in the browser to achieve maximum performance. Note though that your workflows will only be useable on browsers that support WebAssembly (basically everything except Internet Explorer). (Generally speaking, developers who are likely to contribute to an open-source project will have modern browsers.)

If you want to host the Tribble instances for multiple projects in one place, have no fear, that's exactly what workflows are for! You can define as many as you want (as long as they're the same across all locales), and then you can use one Tribble instance for many entirely different projects (or just for different parts of a very large project).

If you visit your deployed Tribble instance and you see an unstyled website (i.e. massive arrows taking up the whole screen), make sure you have the `--path` setting correct in `tribble deploy`. Tribble uses this to know where it is, and to know the location of its CSS files, hence why it appears unstyled when this setting is incorrect.

## License

See [`LICENSE`](./LICENSE).
