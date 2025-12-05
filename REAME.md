# ProcessLint

A distributed, file based process enforcement and linting tool.

Have you heard any of the following at your workplace?

* "Sorry, I named the file wrong. There are two spaces instead of one."
* "Well, the character design was updated, but we forgot to update the character models."
* "I thought these assets were being rebuilt when the sources changed. Aren't they?"
* "The design review signoff sheet is missing!"
* "Oops, I accidentally skipped a number on the sidecar when checking in the final render."
* "Please don't put spaces in filenames. It makes our lives much more difficult!"
* "We're missing unit tests for vector_math.c."
* "The texture was checked in but its size isn't POT."
* "The jira task says has this is marked as complete, but the work isn't where it should be."
* "There was an implicit order definition in the Makefile such that asset A always got built before asset B. I didn't notice when it broke."

ProcessLint is a tool that is meant to formalize the many process related issues tha arise in art, gaming, engineering, software, and other heavily process oriented development workplaces. It aims to address deficiencies in process definition which even today are frequently unspecified or carried around by word-of-mouth, and which frequently contribute to team friction.

In short, the tool helps *define* and *lint* any process which can be represented as a file structure. 

This includes:

* Art/Game development: where designs generate images, which may generate models, which might be packed in archives, and where large teams must name and place everything correctly.
* Engineering: where analyses and code may flow from design documents and knowing the stale status of previous analyses or code is important as designs change.
* Software development: where current linting tools frequency don't address file structure or naming conventions, or such areas may be addressed ad-hoc.

ProcessLint finds places where the current files and artifacts don't adhere to a carefully predefined process, and can do so automatically on change as part of CI.

In other words, ProcessLint:

* Lets you define directory naming and structure conventions.
* Lets you define file naming and placement conventions.
* Lets you define file dependency structures, so missing or stale downstream artifacts can be detected.
* Provides custom hooks for individual file formedness checks.
* Reports the state of a file tree to define how well it adheres to the defined process. Deviations can be returned as warnings or errors.
* Can provide state information to existing database oriented process management systems which might have an imperfect knowledge of the current project state.
* Is meant to be configuration management system agnostic, so can therefore work with Git, SVN etc. The process definitions are just files to be checked in, and no server is necessary (it's distributed)

# Example

Here's a run of the `proclint` tool on a theoretical configuration managed repository:

```
proclint --all

ERROR: file "renders/approved/scene07_1.mp4" is does not adhere to the naming convention for its directory. Did you mean "renders/approved/scene_07_001.mp4" ?
ERROR: directory "models/scene 01" does not adhere to convention. Did you mean "models/scene_01" ?
WARNING: Found file "art/backgrounds/xxx.png" This directory should only contain ".jpg" files. Are you sure this is in the right place?
WARNING: File "art/unapproved/skins/character_001.jpg" is now stale due to the update of art/designs/character_001_design.jpg". Should it be updated?
WARNING: File "code/math/calc.c has no corresponding unit test at "code/unit_tests/math/calc_tests.c"
WARNING: File "models/textures/cube.tiff is reported to be deficient by command "pot_check.sh".
MISSING: File "reports/failure_analysis.xml" is missing.
MISSING: File "reports/cost_analysis.docx" missing and blocked by missing dependency "reports/cost_report.docx"

```

Configuration can define what process deviations are defined as errors, which as warning and the tool returns a corresponding error code for CI integration.

