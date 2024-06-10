from setuptools import setup, find_packages

setup(
    name='vide_tool',
    version='0.1.0',
    packages=find_packages(),
    entry_points={
        'console_scripts': [
            'vide=vide.cli:main',  # "vide" is the command you'll use in the CLI
        ],
    },
    install_requires=[
        # list your project's dependencies here
        # 'numpy',
        # 'requests',
    ],
    # Add other metadata like your project's description, author, etc.
)
