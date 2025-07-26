# Anon Gift Swop 🎁👤🔄

A production-ready static website that creates fair gift exchange assignments with advanced constraint handling and forbidden pairing support.

## Features

- **Forbidden Pairings**: Block specific people from being paired together
- **Assignment History**: Considers previous years to ensure variety over time
- **Fully Client-Side**: No server required - runs entirely in your browser
- **Shareable Links**: Generate unique URLs for each participant
- **Onboarding Flow**: Guided setup for new users

## Frequently Asked Questions

### Why the British spelling `swop`?
The original creator of this project is British. It is also a slight nod to avoid Americentrism.

### Why not `secret santa`?
This is to avoid religious bias.

### Development
```bash
# Install global tools
make setup-tools

# Install dependencies
make install

# Start development server
make dev

# Run tests
make test

# Build for production
make build
```

### Using the Application
1. **Add Participants**: Enter names of people participating in the gift exchange
2. **Set Restrictions**: Define forbidden pairings (e.g., spouses, family members)
3. **Generate & Share**: Create assignments and share unique links with each participant

### Technology Stack
- **Frontend**: SvelteKit with TypeScript
- **Core Logic**: Rust compiled to WebAssembly
- **Testing**: Comprehensive property-based testing with proptest
- **CI/CD**: GitHub Actions with automated testing and deployment

## Contributing

We welcome contributions from everyone! If you’d like to report a bug, suggest an enhancement, or submit a pull request, please ensure your changes align with the project's goals and clearly describe your contributions in your pull request.
Your input is valuable, so feel free to reach out with any questions or ideas. Together, we can make this project even better!

When creating an issue/pull request (PR), please be descriptive! The more information you give us, the better we can understand the problem/proposed changes. Also, make sure to ensure tests pass/add new tests if needed before opening a PR.

