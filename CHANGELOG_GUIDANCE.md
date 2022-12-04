# `CHANGELOG.md` Guidance

At this stage, we manage the changelog manually. Nothing fancy.

Each entry has to match a release, and follow this format:

```markdown
# vMAJOR.MINOR.PATCH (20??-??-??)

## Breaking Changes

## Features

## Enhancements

## Bug Fixes

## Notes
```

The `# H1` should be `version (ISO DATE)`.

The `## H2` are instead categories of what we want to report about this version.
**IMPORTANT:** Before cutting the release, remove any section that is empty for the given release: no point
in publishing empty sections.

## Categorization

Information in each entry should be structured as follows:

`## Breaking Changes`: This section documents in brief any incompatible changes and how to handle them.
**This should only be present in major (or, in some cases, minor) version upgrades**.

`## Features`: These are new improvements and features that deserve to be highlighted.
**This should be marked by a minor version upgrade**.

`## Enhancements`: Smaller features added to the project.

`## Bug Fixes`: Any bugs that were fixed.

`## Notes`: Additional information for potentially unexpected upgrade behavior, notice of upcoming deprecations,
or anything worth highlighting to the user that does not fit in the other categories.
**This should not be abused**: always consider if the information is of any material importance to the user.
