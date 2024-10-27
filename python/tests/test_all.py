import pytest
import md0


def test_sum_as_string():
    assert md0.sum_as_string(1, 1) == "2"
