//
//  ManagedSmartFolder+CoreDataProperties.m
//  8views
//
//  Created by C.W. Betts on 6/23/20.
//  Copyright © 2020 Dancing Tortoise Software. All rights reserved.
//
//

#import "ManagedSmartFolder+CoreDataProperties.h"

@implementation ManagedSmartFolder (CoreDataProperties)

+ (NSFetchRequest<ManagedSmartFolder *> *)fetchRequest {
	return [NSFetchRequest fetchRequestWithEntityName:@"SmartFolder"];
}


@end
