with open("tools/x32_tap/src/main.rs", "r") as f:
    text = f.read()

# Add 2 missing `}` at the end of the `if let Ok(Ok(msg))` block
# We can find `was_above_threshold = false;` and add braces
old = r"""                                was_above_threshold = false;
                            }
                        }
                    }
                }
            }
        }
    } else {"""
new = r"""                                was_above_threshold = false;
                            }
                        }
                    }
                }
            }
        }
    } else {"""

text = text.replace(old, new)
with open("tools/x32_tap/src/main.rs", "w") as f:
    f.write(text)
