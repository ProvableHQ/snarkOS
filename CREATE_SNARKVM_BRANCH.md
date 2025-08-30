# Commands to create snarkvm-update branch and draft PR

# These commands should be run by someone with push access to the repository:

# 1. Create and push the snarkvm-update branch based on staging
git fetch origin staging
git checkout -b snarkvm-update origin/staging
git push origin snarkvm-update

# 2. Create a draft PR using GitHub CLI or web interface with:
# - Title: "[Do Not Merge] snarkVM CI check"
# - Base: staging
# - Head: snarkvm-update
# - Draft: true

# Using GitHub CLI:
gh pr create --title "[Do Not Merge] snarkVM CI check" --body "This PR is used for automated snarkVM dependency updates. It should remain as a draft and not be merged." --base staging --head snarkvm-update --draft

# The workflow will automatically:
# - Check for new commits in snarkVM staging every hour
# - Update Cargo.toml with the latest commit hash when changes are detected
# - Commit and push changes to the snarkvm-update branch