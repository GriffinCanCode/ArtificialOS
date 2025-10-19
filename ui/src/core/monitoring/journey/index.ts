/**
 * Journey Tracking Subsystem
 * End-to-end user interaction tracking across windows and apps
 */

export { useJourneyStore, journeyStore } from './store';
export { JourneyProvider, WindowJourneyProvider, AppJourneyProvider, FormJourneyProvider, useJourneyContext, withJourneyTracking, type WindowJourneyProviderProps, type AppJourneyProviderProps, type FormJourneyProviderProps } from './providers';
export type { Journey, JourneyStep, JourneyStepType, JourneyStepContext, JourneyConfig, JourneyAnalytics, WindowJourney, JourneyPerformance, JourneyClassification, JourneyPattern, JourneyOutcome, JourneyExperience } from './types';

