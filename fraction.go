package main

import (
	"fmt"
	"strings"
)

// my_gcd calculates the Greatest Common Divisor of two integers.
func my_gcd(a, b int) int {
	for b != 0 {
		a, b = b, a%b
	}
	return a
}

// FractionToLilypond converts a fraction to LilyPond duration strings.
func FractionToLilypond(numerator int, denominator int) []string {
	if denominator == 0 {
		return []string{"Invalid denominator"}
	}

	lilypondMap := map[string]string{
		"1/1":   "1",
		"1/2":   "2",
		"1/4":   "4",
		"1/8":   "8",
		"1/16":  "16",
		"1/32":  "32",
		"1/64":  "64",
		"1/128": "128",
		"3/2":   "1.",
		"3/4":   "2.",
		"3/8":   "4.",
		"3/16":  "8.",
		"3/32":  "16.",
		"3/64":  "32.",
		"3/128": "64.",
		"7/4":   "1..",
		"7/8":   "2..",
		"7/16":  "4..",
		"7/32":  "8..",
		"7/64":  "16..",
		"7/128": "32..",
	}

	fractionStr := fmt.Sprintf("%d/%d", numerator, denominator)
	if lilypondDuration, ok := lilypondMap[fractionStr]; ok {
		return []string{lilypondDuration}
	}

	result := []string{}
	remainingNumerator := numerator
	remainingDenominator := denominator

	commonDenominators := []int{1, 2, 4, 8, 16, 32, 64, 128}

	// Loop Detection
	seenFractions := make(map[string]bool)

	for remainingNumerator > 0 {
		currentFraction := fmt.Sprintf("%d/%d", remainingNumerator, remainingDenominator)
		if seenFractions[currentFraction] {
			// Loop detected! Fallback to ties
			return tieFallback(numerator, denominator, commonDenominators)
		}
		seenFractions[currentFraction] = true

		bestDenominator := -1
		for i := len(commonDenominators) - 1; i >= 0; i-- {
			denom := commonDenominators[i]
			if remainingNumerator*denom <= remainingDenominator {
				bestDenominator = denom
				break
			}
		}

		if bestDenominator == -1 {
			return []string{fmt.Sprintf("Complex: %d/%d", numerator, denominator)}
		}

		bestNum := remainingNumerator * bestDenominator / remainingDenominator
		result = append(result, fmt.Sprintf("%d", remainingDenominator/bestDenominator))
		remainingNumerator = remainingNumerator*bestDenominator - bestNum*remainingDenominator
		remainingDenominator = remainingDenominator * bestDenominator

		// ---  SIMPLIFY ---
		common := my_gcd(remainingNumerator, remainingDenominator)
		remainingNumerator /= common
		remainingDenominator /= common
	}

	// Add ties (if the main logic succeeded)
	tiedResult := []string{}
	for i, note := range result {
		tiedResult = append(tiedResult, note)
		if i < len(result)-1 {
			tiedResult = append(tiedResult, "~")
		}
	}

	return tiedResult
}

// tieFallback decomposes the fraction into the smallest possible notes and ties them.
func tieFallback(numerator int, denominator int, commonDenominators []int) []string {
	result := []string{}
	for numerator > 0 {
		result = append(result, fmt.Sprintf("%d", denominator))
		numerator--
	}

	tiedResult := []string{}
	for i, note := range result {
		tiedResult = append(tiedResult, note)
		if i < len(result)-1 {
			tiedResult = append(tiedResult, "~")
		}
	}
	return tiedResult
}

func main_test() {
	fractions := [][]int{
		{5, 32},
		{7, 12},
		{11, 16},
		{2, 3},
		{5, 64},
		{7, 64},
		{5, 32}, // Test case that was looping
	}

	for _, frac := range fractions {
		result := FractionToLilypond(frac[0], frac[1])
		fmt.Printf("%d/%d  =>  %s\n", frac[0], strings.Join(result, " "))
	}
}
