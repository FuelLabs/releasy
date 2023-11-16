# Releasy

Releasy is our release automation tool designed for helping us with our release process at Fuel Labs.

## Usage 

Releasy requires a `repo-plan.toml` file to describe repository relations for correctly handling tracking branches across repos. An example for a `repo-plan.toml` which declares the releation between `fuel-core` and `fuels-rs` can be seen below.

```TOML
[current-repo]
name = "fuels-rs"
owner = "FuelLabs"

[repo.fuels-rs.details]
name = "fuels-rs"
owner = "FuelLabs"

[repo.fuel-core.details]
name = "fuel-core"
owner = "FuelLabs"

[repo.fuels-rs]
dependencies = ["fuel-core"]
```

After placing repo description file into the repo, we need to add releasy ci jobs so that tracking branches are updated.

### Dependency Commmits

Handles tracking branch updates in case there is a commit to repository dependended by the current repo.

```yml
on:
  repository_dispatch:
    types: [new-commit-to-dependency]

jobs:
  new_commit_to_dependency:
    runs-on: ubuntu-latest
    env:
      GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
    steps:
    - uses: actions/checkout@v4
    - run: |
        cargo install releasy-handler
        releasy-handler --event new-commit-to-dependency --event-repo-name ${{ github.event.client_payload.repo.name }} --event-repo-owner ${{ github.event.client_payload.repo.owner }} --event-commit-hash ${{ github.event.client_payload.details.commit_hash}} --path ./.github/workflows/repo-plan.toml
```

### Self Commits

Handles tracking branch updates in case there is a commit to repository depended by the current repo.

```yml
on:
  push:
    branches:
    - master

jobs:
  new_commit_to_self:
    runs-on: ubuntu-latest
    env:
      GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
    steps:
    - uses: actions/checkout@v4
    - run: |
        cargo install --path releasy-handler
        releasy-handler --event new-commit-to-self --event-repo-name fuels-rs --event-repo-owner FuelLabs --path ./.github/workflows/repo-plan.toml --event-commit-hash ${GITHUB_SHA}
```

### Notify Downstream Repos 

Notifying downstream repos for a commit to current repo.


```yml
name: Notify downstream repos

on:
  push:
    branches:
    - master

jobs:

  notify:
    runs-on: ubuntu-latest
    env:
      DISPATCH_TOKEN: ${{ secrets.DISPATCH_TOKEN }} 
    steps:
    - uses: actions/checkout@v4
    - run: |
        cargo install --path releasy-emit
        releasy-emit --event new-commit-to-dependency --path ./.github/workflows/repo-plan.toml --event-commit-hash ${GITHUB_SHA}
```
